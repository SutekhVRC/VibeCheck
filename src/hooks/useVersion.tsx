import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import { createToast } from "../components/Toast";
import { VERSION } from "../data/constants";

export function useVersion() {
  const [version, setVersion] = useState("");

  async function getVersion() {
    try {
      const version = await invoke<string>(VERSION);
      setVersion(version);
    } catch (e) {
      createToast("error", "Could not get version!", JSON.stringify(e));
    }
  }
  useEffect(() => {
    getVersion();
  }, []);
  return { version };
}
