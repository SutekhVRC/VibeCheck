import { invoke } from "@tauri-apps/api";
import { VERSION } from "../data/constants";
import { useEffect, useState } from "react";
import { useToastContext } from "../context/ToastContext";

export function useVersion() {
  const [version, setVersion] = useState("");
  const toast = useToastContext();

  async function getVersion() {
    try {
      const version = await invoke<string>(VERSION);
      setVersion(version);
    } catch (e) {
      toast.createToast("Could not get version!", JSON.stringify(e), "error");
    }
  }
  useEffect(() => {
    getVersion();
  }, []);
  return { version };
}
