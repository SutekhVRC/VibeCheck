import { listen } from "@tauri-apps/api/event";
import { createContext, useContext, useEffect, useState } from "react";
import { assertExhaustive } from "../utils";
import { TOY_EVENT } from "../data/constants";
import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import type { FeToyEvent } from "../../src-tauri/bindings/FeToyEvent";
import type { ReactNode } from "react";

export type ToyMap = {
  [id: number]: FeVCToy;
};

type ToyContext = {
  toys: ToyMap;
};

const EMPTY_TOY_MAP = {} as ToyMap;

const ToysContext = createContext<ToyContext>({
  toys: EMPTY_TOY_MAP,
});

export function useToys() {
  return useContext(ToysContext);
}

export function ToysProvider({ children }: { children: ReactNode }) {
  const [toys, setToys] = useState<ToyMap>(EMPTY_TOY_MAP);

  useEffect(() => {
    const unlistenPromise = listen<FeToyEvent>(TOY_EVENT, (event) => {
      switch (event.payload.kind) {
        case "Add":
          const add = event.payload.data;
          setToys((t) => {
            return {
              ...t,
              [add.toy_id]: add,
            };
          });
          break;
        case "Remove":
          const remove = event.payload.data;
          setToys((t) => {
            const { [remove]: _, ...newToys } = t;
            return newToys;
          });
          break;
        case "Update":
          const update = event.payload.data;
          setToys((t) => {
            return {
              ...t,
              [update.toy_id]: update,
            };
          });
          break;
        default:
          assertExhaustive(event.payload);
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    // If any toy has batery_level == 0, keep re-requesting every second
    const some_toy_has_zero_battery = Object.values(toys).reduce((acc, e) => {
      return acc || e.battery_level == 0.0;
    }, false);
    if (!some_toy_has_zero_battery || Object.keys(toys).length == 0) return;
    const t = setInterval(() => {
      // force_toy_update?
    }, 1000);
    return () => clearInterval(t);
  }, [toys]);

  return (
    <ToysContext.Provider value={{ toys }}>{children}</ToysContext.Provider>
  );
}
