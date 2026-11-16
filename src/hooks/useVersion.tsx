import { errorText } from "@/lib/errors";
import { getName } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { INVOKE } from "../data/constants";

export function useVersion() {
  const [version, setVersion] = useState("");
  // The app's product name from its tauri.conf.json
  const [appName, setAppName] = useState("VibeCheck");

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
    getName()
      .then(setAppName)
      .catch(() => {});
  }, []);
  return { version, appName };
}
