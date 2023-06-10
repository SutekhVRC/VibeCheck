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

export function useToys() {
  const [toys, setToys] = useState<ToyMap>({});

  useEffect(() => {
    async function getOfflinetoys() {
      try {
        const offlineToys = await invoke<FeVCToy[]>(OFFLINE_SYNC);
        setToys(
          offlineToys.reduce((acc, val) => {
            acc[`${val.toy_name} ${val.sub_id}`] = val;
            return acc;
          }, {} as ToyMap)
        );
      } catch (e) {
        createToast("error", "Could not load offline toys", JSON.stringify(e));
      }
    }
    getOfflinetoys();
  }, []);

  function handleToyEvent(payload: FeToyEvent): void {
    switch (payload.kind) {
      case "Add":
        setToys((toys) => {
          const {
            // eslint-disable-next-line @typescript-eslint/no-unused-vars
            [`${payload.data.toy_name} 255`]: _,
            ...toysWithOfflineCopyRemoved
          } = toys;
          return {
            ...toysWithOfflineCopyRemoved,
            [`${payload.data.toy_name} ${payload.data.sub_id}`]: payload.data,
          };
        });
        break;
      case "Update":
        setToys((toys) => {
          return {
            ...toys,
            [`${payload.data.toy_name} ${payload.data.sub_id}`]: payload.data,
          };
        });
        break;
      case "Remove":
        setToys((toys) => {
          const onlineToOfflineToy = Object.values(toys).find(
            (t) => t.toy_id == payload.data
          );
          if (onlineToOfflineToy == undefined) {
            createToast(
              "warn",
              "Remove Toy",
              `Could not find toy with id ${payload.data}`
            );
            return toys;
          }
          const {
            // eslint-disable-next-line @typescript-eslint/no-unused-vars
            [`${onlineToOfflineToy.toy_name} ${onlineToOfflineToy.sub_id}`]: _,
            ...newToys
          } = toys;
          onlineToOfflineToy.toy_id = null;
          onlineToOfflineToy.sub_id = 255;
          onlineToOfflineToy.toy_connected = false;
          onlineToOfflineToy.battery_level = null;
          return {
            ...newToys,
            [`${onlineToOfflineToy.toy_name} 255`]: onlineToOfflineToy,
          };
        });
        break;
      default:
        assertExhaustive(payload);
    }
  }

  async function handleToyAlter(newToy: FeVCToy) {
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

  function handleFeatureAlter(newToy: FeVCToy, newFeature: FeVCToyFeature) {
    const newFeatures = [...newToy.features];
    newFeatures[newFeature.feature_index] = newFeature;
    handleToyAlter({ ...newToy, features: newFeatures });
  }

  useEffect(() => {
    const unlistenPromise = listen<FeToyEvent>(TOY_EVENT, (event) =>
      handleToyEvent(event.payload)
    );

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  return { toys, handleToyAlter, handleFeatureAlter };
}
