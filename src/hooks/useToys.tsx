import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { ALTER_TOY, OFFLINE_SYNC, TOY_EVENT } from "../data/constants";
import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import type { FeToyEvent } from "../../src-tauri/bindings/FeToyEvent";
import { assertExhaustive } from "../utils";
import { createToast } from "../components/Toast";
import { invoke } from "@tauri-apps/api";
import { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";

type ToyMap = {
  [id: string]: FeVCToy;
};

export async function handleToyAlter(newToy: FeVCToy) {
  try {
    if (newToy.toy_connected) {
      await invoke(ALTER_TOY, {
        mutate: { Connected: newToy },
      });
    } else {
      await invoke(ALTER_TOY, {
        mutate: { Disconnected: newToy },
      });
    }
  } catch (e) {
    createToast("error", "Could not alter toy!", JSON.stringify(e));
  }
}

export async function handleFeatureAlter(
  newToy: FeVCToy,
  newFeature: FeVCToyFeature
) {
  const newFeatures = [...newToy.features];
  newFeatures[newFeature.feature_index] = newFeature;
  await handleToyAlter({ ...newToy, features: newFeatures });
}

export function useToys() {
  const [offlineToys, setOfflineToys] = useState<ToyMap>({});
  const [onlineToys, setOnlineToys] = useState<ToyMap>({});
  const toys = {} as ToyMap;
  const onlineToyNames = new Set();
  Object.values(onlineToys).forEach((t) => {
    onlineToyNames.add(t.toy_name);
    toys[`${t.toy_name} ${t.sub_id}`] = t;
  });
  Object.values(offlineToys).forEach((t) => {
    if (!onlineToyNames.has(t.toy_name)) toys[`${t.toy_name} ${t.sub_id}`] = t;
  });

  async function getOfflinetoys() {
    try {
      const offlineToys = await invoke<FeVCToy[]>(OFFLINE_SYNC);
      setOfflineToys(
        offlineToys.reduce((acc, val) => {
          acc[`${val.toy_name} ${val.sub_id}`] = val;
          return acc;
        }, {} as ToyMap)
      );
    } catch (e) {
      createToast("error", "Could not load offline toys", JSON.stringify(e));
    }
  }

  useEffect(() => {
    getOfflinetoys();
  }, []);

  function handleToyEvent(payload: FeToyEvent): void {
    switch (payload.kind) {
      case "Add":
        setOnlineToys((curOnlineToys) => {
          return {
            ...curOnlineToys,
            [`${payload.data.toy_name} ${payload.data.sub_id}`]: payload.data,
          };
        });
        break;
      case "Update":
        if (payload.data.toy_connected) {
          setOnlineToys((curOnlineToys) => {
            return {
              ...curOnlineToys,
              [`${payload.data.toy_name} ${payload.data.sub_id}`]: payload.data,
            };
          });
        } else {
          setOfflineToys((curOfflineToys) => {
            return {
              ...curOfflineToys,
              [`${payload.data.toy_name} ${payload.data.sub_id}`]: payload.data,
            };
          });
        }

        break;
      case "Remove":
        setOnlineToys((curOnlineToys) => {
          const filtered = Object.values(curOnlineToys).filter(
            (t) => t.toy_id != payload.data
          );
          return filtered.reduce((acc, val) => {
            acc[`${val.toy_name} ${val.sub_id}`] = val;
            return acc;
          }, {} as ToyMap);
        });
        getOfflinetoys(); // Ensure sync
        break;
      default:
        assertExhaustive(payload);
    }
  }

  useEffect(() => {
    const unlistenPromise = listen<FeToyEvent>(TOY_EVENT, (event) =>
      handleToyEvent(event.payload)
    );

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  return { toys };
}
