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
      // TODO figure out a less dumb way to do this
      const { FeToyAdd: addPayload } = event.payload as { FeToyAdd: FeVCToy };
      const { FeToyRemove: removePayload } = event.payload as {
        FeToyRemove: number;
      };
      if (addPayload != undefined) {
        setToys((t) => {
          return {
            ...t,
            [addPayload.toy_id]: addPayload,
          };
        });
      }
      if (removePayload != undefined) {
        setToys((t) => {
          const { [removePayload]: _, ...newToys } = t;
          return newToys;
        });
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => {
        unlisten();
      });
    };
  }, []);

  return (
    <ToysContext.Provider value={{ toys, refetchToys }}>
      {children}
    </ToysContext.Provider>
  );
}
