import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import type { FeToyEvent } from "../../src-tauri/bindings/FeToyEvent";
import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import type { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";
import { INVOKE, LISTEN } from "../data/constants";
import { assertExhaustive } from "../utils";

type ToyMap = {
  [id: string]: FeVCToy;
};

export async function handleToyAlter(newToy: FeVCToy) {
  try {
    if (newToy.toy_connected) {
      await invoke(INVOKE.ALTER_TOY, {
        mutate: { Connected: newToy },
      });
    } else {
      await invoke(INVOKE.ALTER_TOY, {
        mutate: { Disconnected: newToy },
      });
    }
  } catch (e) {
    toast.error(`Could not alter toy!\n${JSON.stringify(e)}`);
  }
}

export async function handleFeatureAlter(
  newToy: FeVCToy,
  newFeature: FeVCToyFeature,
) {
  const newFeatures = [...newToy.features];
  // We need to find the array index because feature_index is not unique
  // And it is completely separate from the array index
  const newFeatureArrayIndex = newFeatures
    .map((f, i) => {
      return {
        arrayIndex: i,
        feature_type: f.feature_type,
        feature_index: f.feature_index,
      };
    })
    .find(
      (f) =>
        f.feature_index == newFeature.feature_index &&
        f.feature_type == newFeature.feature_type,
    )?.arrayIndex;
  if (newFeatureArrayIndex == null) return; // newFeature [type + index] does not exist in feature array
  newFeatures[newFeatureArrayIndex] = newFeature;
  await handleToyAlter({ ...newToy, features: newFeatures });
}

export function parseName(s: string) {
  return s.replace("Lovense Connect ", "Lovense ");
}

export function toyKey(t: FeVCToy) {
  return `${parseName(t.toy_name)} ${t.sub_id}`;
}

export function useToys() {
  const [offlineToys, setOfflineToys] = useState<ToyMap>({});
  const [onlineToys, setOnlineToys] = useState<ToyMap>({});
  const toys = {} as ToyMap;
  const onlineToyNames = new Set();
  Object.values(onlineToys).forEach((t) => {
    const name = parseName(t.toy_name);
    onlineToyNames.add(name);
    toys[toyKey(t)] = t;
  });
  Object.values(offlineToys).forEach((t) => {
    const name = parseName(t.toy_name);
    if (!onlineToyNames.has(name)) toys[toyKey(t)] = t;
  });

  async function syncOfflineToys() {
    try {
      const offlineToys = await invoke<FeVCToy[]>(INVOKE.OFFLINE_SYNC, {
        refreshToys: true,
      });
      setOfflineToys(
        offlineToys.reduce((acc, val) => {
          acc[toyKey(val)] = val;
          return acc;
        }, {} as ToyMap),
      );
    } catch (e) {
      toast.error(`Could not load offline toys\n${JSON.stringify(e)}`);
    }
  }

  useEffect(() => {
    syncOfflineToys();
  }, []);

  async function handleToyEvent(payload: FeToyEvent) {
    switch (payload.kind) {
      case "Add":
        setOnlineToys((curOnlineToys) => {
          return {
            ...curOnlineToys,
            [toyKey(payload.data)]: payload.data,
          };
        });
        break;
      case "Update":
        if (payload.data.toy_connected) {
          setOnlineToys((curOnlineToys) => {
            return {
              ...curOnlineToys,
              [toyKey(payload.data)]: payload.data,
            };
          });
        } else {
          setOfflineToys((curOfflineToys) => {
            return {
              ...curOfflineToys,
              [toyKey(payload.data)]: payload.data,
            };
          });
        }

        break;
      case "Remove":
        await syncOfflineToys();
        setOnlineToys((curOnlineToys) => {
          const filtered = Object.values(curOnlineToys).filter(
            (t) => t.toy_id != payload.data,
          );
          return filtered.reduce((acc, val) => {
            acc[toyKey(val)] = val;
            return acc;
          }, {} as ToyMap);
        });
        break;
      default:
        assertExhaustive(payload);
    }
  }

  useEffect(() => {
    const unlistenPromise = listen<FeToyEvent>(LISTEN.TOY_EVENT, (event) =>
      handleToyEvent(event.payload),
    );

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  return { toys, hasOnlineToys: Object.values(onlineToys).length > 0 };
}
