import { invoke } from "@tauri-apps/api";
import { VERSION } from "../data/constants";
import { useEffect, useState } from "react";

export function useVersion() {
  const [version, setVersion] = useState("");
  async function getVersion() {
    try {
      const version = await invoke<string>(VERSION);
      setVersion(version);
    } catch (e) {
      alert(e);
    }
  }
  useEffect(() => {
    getVersion();
  }, []);
  return { version };
}
