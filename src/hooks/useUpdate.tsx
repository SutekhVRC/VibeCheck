import { checkUpdate } from "@tauri-apps/api/updater";
import { useEffect, useState } from "react";

export function useUpdate() {
  const [canUpdate, setCanUpdate] = useState(false);
  useEffect(() => {
    checkUpdate().then((res) => setCanUpdate(res.shouldUpdate));
  }, []);
  return { canUpdate };
}
