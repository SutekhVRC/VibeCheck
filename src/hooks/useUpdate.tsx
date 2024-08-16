import { Update, check } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";

export function useUpdate() {
  const [canUpdate, setCanUpdate] = useState(false);
  useEffect(() => {
    check().then((res: Update | null) => setCanUpdate(res?.available ?? false));
  }, []);
  return { canUpdate };
}
