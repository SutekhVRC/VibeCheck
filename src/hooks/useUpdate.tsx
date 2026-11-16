import { INVOKE } from "@/data/constants";
import { invoke } from "@tauri-apps/api/core";
import { check } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";

/**
 * - checking: not known yet
 * - available: an update can be installed
 * - current: no newer version
 * - failed: the check didn't work (e.g. offline)
 * - off: this build doesn't check for updates
 */
export type UpdateStatus =
  "checking" | "available" | "current" | "failed" | "off";

export function useUpdate() {
  const [status, setStatus] = useState<UpdateStatus>("checking");
  useEffect(() => {
    async function fetchUpdate() {
      const enabled = await invoke<boolean>(INVOKE.UPDATER_ENABLED).catch(
        () => true,
      );
      if (!enabled) {
        setStatus("off");
        return;
      }
      try {
        const update = await check();
        setStatus(update !== null ? "available" : "current");
      } catch (e) {
        console.warn("Couldn't check for updates:", e);
        setStatus("failed");
      }
    }
    fetchUpdate();
  }, []);
  return { canUpdate: status == "available", updateStatus: status };
}
