import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { OFFLINE_SYNC, TOY_EVENT } from "../data/constants";
import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import type { FeToyEvent } from "../../src-tauri/bindings/FeToyEvent";
import { assertExhaustive } from "../utils";
import { useToastContext } from "../context/ToastContext";
import { invoke } from "@tauri-apps/api";

export type OnlineToyMap = {
  [id: number]: FeVCToy;
};
export type OfflineToyMap = {
  [id: string]: FeVCToy;
};

export function useToys() {
  const [onlineToys, setOnlineToys] = useState<OnlineToyMap>({});
  const [offlineToys, setOfflineToys] = useState<OfflineToyMap>({});
  const toast = useToastContext();

  useEffect(() => {
    async function getOfflinetoys() {
      try {
        const offlineToys = await invoke<FeVCToy[]>(OFFLINE_SYNC);
        setOfflineToys(
          offlineToys.reduce((acc, val) => {
            acc[val.toy_name] = val;
            return acc;
          }, {} as OfflineToyMap)
        );
      } catch (e) {
        toast.createToast(
          "Could not load offline toys",
          JSON.stringify(e),
          "error"
        );
      }
    }
    getOfflinetoys();
  }, []);

  function handleToyEvent(payload: FeToyEvent): void {
    switch (payload.kind) {
      case "Add":
        if (!payload.data.toy_connected) {
          // TODO I don't think this case ever happens?
          toast.createToast(
            "Add toy",
            "Adding toy that is not connected?",
            "error"
          );
          return;
        }
        setOfflineToys((t) => {
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          const { [payload.data.toy_name]: _, ...newToys } = t;
          return newToys;
        });
        setOnlineToys((t) => {
          return {
            ...t,
            [payload.data.toy_id as number]: payload.data,
          };
        });
        break;
      case "Update":
        if (payload.data.toy_id != null) {
          setOnlineToys((t) => {
            return {
              ...t,
              [payload.data.toy_id as number]: payload.data,
            };
          });
        } else {
          setOfflineToys((t) => {
            return {
              ...t,
              [payload.data.toy_name]: payload.data,
            };
          });
        }
        break;
      case "Remove":
        // TODO get rid of ugly nested setState?
        setOnlineToys((t) => {
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          const { [payload.data]: onlineToOfflineToy, ...newToys } = t;
          onlineToOfflineToy.toy_id = null;
          onlineToOfflineToy.sub_id = 255;
          onlineToOfflineToy.toy_connected = false;
          onlineToOfflineToy.battery_level = null;
          setOfflineToys((t) => {
            return {
              ...t,
              [onlineToOfflineToy.toy_name]: onlineToOfflineToy,
            };
          });
          return newToys;
        });
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

  return { onlineToys, offlineToys };
}
