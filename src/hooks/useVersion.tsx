import { invoke } from "@tauri-apps/api";
import { VERSION } from "../data/constants";
import { useEffect, useState } from "react";

export function useVersion() {
  const [version, setVersion] = useState("");
  async function getVersion() {
    await invoke<string>(VERSION).then((v:string) => setVersion(v));
  }
  useEffect(() => {
    getVersion();
  }, []);
  return { version };
}
