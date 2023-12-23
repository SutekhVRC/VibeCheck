import { relaunch } from "@tauri-apps/api/process";
import { installUpdate } from "@tauri-apps/api/updater";
import Button from "../layout/Button";
import { createToast } from "./Toast";
import UpdatePing from "./UpdatePing";

export default function UpdateButton({ enabled }: { enabled: boolean }) {
  async function handleUpdate() {
    try {
      await installUpdate();
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
