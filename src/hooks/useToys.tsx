import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { TOY_EVENT } from "../data/constants";
import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import type { FeToyEvent } from "../../src-tauri/bindings/FeToyEvent";
import { assertExhaustive } from "../utils";

export type ToyMap = {
  [id: number]: FeVCToy;
};

export function useToys() {
  const [toys, setToys] = useState<ToyMap>([]);

  function handleToyEvent(payload: FeToyEvent): void {
    switch (payload.kind) {
      case "Add":
        setToys((t) => {
          return {
            ...t,
            [payload.data.toy_id]: payload.data,
          };
        });
        break;
      case "Remove":
        setToys((t) => {
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          const { [payload.data]: _, ...newToys } = t;
          return newToys;
        });
        break;
      case "Update":
        setToys((t) => {
          return {
            ...t,
            [payload.data.toy_id]: payload.data,
          };
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

  return { toys };
}
