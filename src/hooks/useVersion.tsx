import { errorText } from "@/lib/errors";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { INVOKE } from "../data/constants";

export function useVersion() {
  const [version, setVersion] = useState("");

  async function getVersion() {
    try {
      const version = await invoke<string>(INVOKE.VERSION);
      setVersion(version);
    } catch (e) {
      toast.error(`Could not get version!\n${errorText(e)}`);
    }
  }
  useEffect(() => {
    getVersion();
  }, []);
  return { version };
}
