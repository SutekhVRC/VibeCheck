import { UpdateResult, check } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";

export function useUpdate() {
  const [canUpdate, setCanUpdate] = useState(false);
  useEffect(() => {
    check().then((res: UpdateResult) => setCanUpdate(res.shouldUpdate));
  }, []);
  return { canUpdate };
}
