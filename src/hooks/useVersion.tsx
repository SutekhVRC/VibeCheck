import { invoke } from "@tauri-apps/api";
import { VERSION } from "../data/constants";
import { useEffect, useState } from "react";

export function useVersion() {
  const [version, setVersion] = useState("");
  async function getVersion() {
    await invoke<string>(VERSION).then((r) => {
      setVersion(r.replaceAll("-", " "));
    });
  }
  useEffect(() => {
    getVersion();
  }, []);
  return { version };
}
