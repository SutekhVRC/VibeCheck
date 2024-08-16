import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import Button from "../layout/Button";
import { createToast } from "./Toast";
import UpdatePing from "./UpdatePing";

export default function UpdateButton({ enabled }: { enabled: boolean }) {
  async function handleUpdate() {
    try {
      const checkUpdate = await check();
      checkUpdate?.downloadAndInstall();
      await relaunch();
    } catch (e) {
      createToast("error", "Could not update!", JSON.stringify(e));
    }
  }
  return (
    <UpdatePing canUpdate={enabled}>
      <Button disabled={!enabled} onClick={handleUpdate}>
        Upgrade
      </Button>
    </UpdatePing>
  );
}
