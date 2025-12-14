import { check } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";

export function useUpdate() {
  const [canUpdate, setCanUpdate] = useState(false);
  useEffect(() => {
    async function fetchUpdate() {
      const update = await check();
      setCanUpdate(update !== null);
    }
    fetchUpdate();
  }, []);
  return { canUpdate };
}
