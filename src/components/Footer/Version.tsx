import { invoke } from "@tauri-apps/api";
import { VERSION } from "../../data/constants";
import { useEffect, useState } from "react";
import { checkUpdate, installUpdate } from "@tauri-apps/api/updater";
import { relaunch } from "@tauri-apps/api/process";

export default function () {
  const [version, setVersion] = useState("");
  const [canUpdate, setCanUpdate] = useState(false);

  async function getVersion() {
    await invoke<string>(VERSION).then((r) => {
      setVersion(r.replaceAll("-", " "));
    });
  }

  async function handleUpdate() {
    if (!canUpdate) return;
    try {
      await installUpdate();
      await relaunch();
    } catch (e) {
      console.log(e);
    }
  }

  useEffect(() => {
    getVersion();
    checkUpdate().then((res) => setCanUpdate(res.shouldUpdate));
  }, []);

  return (
    <div className="flex items-center justify-center">
      <div
        className={`relative select-none ${canUpdate && "cursor-pointer"}`}
        onClick={handleUpdate}
      >
        <div className="pl-3 pr-3">{version}</div>
        {canUpdate && (
          <>
            <div className="absolute top-0 right-0 w-2 h-2 rounded-full bg-green-300 animate-ping" />
            <div className="absolute top-0 right-0 w-2 h-2 rounded-full bg-green-300" />
          </>
        )}
      </div>
    </div>
  );
}
