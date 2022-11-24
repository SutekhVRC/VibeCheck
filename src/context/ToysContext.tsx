import { invoke } from "@tauri-apps/api";
import { createContext, ReactNode, useContext, useState } from "react";
import { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import { GET_TOYS } from "../data/constants";

const ToysContext = createContext<{ toys: FeVCToy[]; refetchToys: () => void }>(
  { toys: [], refetchToys: () => null }
);

export function useToys() {
  return useContext(ToysContext);
}

export function ToysProvider({ children }: { children: ReactNode }) {
  const [toys, setToys] = useState<FeVCToy[]>([]);

  async function refetchToys() {
    await invoke<null | { [key: number]: FeVCToy }>(GET_TOYS).then((response) =>
      setToys(response ? Object.values(response) : [])
    );
  }

  return (
    <ToysContext.Provider value={{ toys, refetchToys }}>
      {children}
    </ToysContext.Provider>
  );
}
