import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { TOY_EVENT } from "../data/constants";
import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import type { FeToyEvent } from "../../src-tauri/bindings/FeToyEvent";
import { assertExhaustive } from "../utils";
import assert from "assert";

export type OnlineToyMap = {
  [id: number]: FeVCToy;
};
export type OfflineToyMap = {
  [id: string]: FeVCToy;
};

export function useToys() {
  const [onlineToys, setOnlineToys] = useState<OnlineToyMap>({});
  const [offlineToys, setOfflineToys] = useState<OfflineToyMap>({});

  function handleToyEvent(payload: FeToyEvent): void {
    switch (payload.kind) {
      case "Add":
        if (payload.data.toy_id == null) {
          // TODO I don't think this case ever happens?
          console.log("ADD WITH NULL TOY ID???");
          assert(false);
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
      case "Remove":
        setOnlineToys((t) => {
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          const { [payload.data]: removedToy, ...newToys } = t;
          // TODO eww this is disgusting
          setOfflineToys((t) => {
            removedToy.toy_id = null;
            removedToy.sub_id = 255;
            removedToy.toy_connected = false;
            removedToy.battery_level = null;
            return {
              ...t,
              [removedToy.toy_name]: removedToy,
            };
          });
          return newToys;
        });
        break;
      case "Update":
        if (payload.data.toy_id == null) {
          // TODO I don't think this case ever happens?
          console.log("REMOVE WITH NULL TOY ID???");
          assert(false);
          return;
        }
        setOnlineToys((t) => {
          return {
            ...t,
            [payload.data.toy_id as number]: payload.data,
          };
        });
        break;
      case "OfflineSyncAll":
        console.log("OFFLINE SYNC");
        setOfflineToys(
          payload.data.reduce((acc, val) => {
            acc[val.toy_name] = val;
            return acc;
          }, {} as OfflineToyMap)
        );
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
