import { check } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";

export function useUpdate() {
  const [canUpdate, setCanUpdate] = useState(false);
  useEffect(() => {
    async function check_update() {
      const update = await check();
      if (update?.available) {
        setCanUpdate(true);
      }
    }
    check_update();
  }, []);
  return { canUpdate };
}
