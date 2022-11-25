import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import {
  createContext,
  ReactNode,
  useContext,
  useEffect,
  useState,
} from "react";
import { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import { FeToyEvent } from "../../src-tauri/bindings/FeToyEvent";
import { GET_TOYS, TOY_EVENT } from "../data/constants";
import { assertExhaustive } from "../utils";

export type ToyMap = {
  [id: number]: FeVCToy;
};

type ToyContext = {
  toys: ToyMap;
  refetchToys: () => void;
};

const EMPTY_TOY_MAP = {} as ToyMap;

const ToysContext = createContext<ToyContext>({
  toys: EMPTY_TOY_MAP,
  refetchToys: () => null,
});

export function useToys() {
  return useContext(ToysContext);
}

export function ToysProvider({ children }: { children: ReactNode }) {
  const [toys, setToys] = useState<ToyMap>(EMPTY_TOY_MAP);

  async function refetchToys() {
    await invoke<null | ToyMap>(GET_TOYS).then((response) =>
      setToys(response ? response : EMPTY_TOY_MAP)
    );
  }

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
        default:
          assertExhaustive(event.payload);
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  return (
    <ToysContext.Provider value={{ toys, refetchToys }}>
      {children}
    </ToysContext.Provider>
  );
}
