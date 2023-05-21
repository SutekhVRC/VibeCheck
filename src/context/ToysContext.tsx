import { listen } from "@tauri-apps/api/event";
import { createContext, useContext, useEffect, useState } from "react";
import { TOY_EVENT } from "../data/constants";
import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import type { FeToyEvent } from "../../src-tauri/bindings/FeToyEvent";
import type { ReactNode } from "react";
import { assertExhaustive } from "../utils";

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

  return (
    <ToysContext.Provider value={{ toys }}>{children}</ToysContext.Provider>
  );
}
