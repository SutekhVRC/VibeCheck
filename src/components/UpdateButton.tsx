import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { toast } from "sonner";
import Button from "../layout/Button";
import UpdatePing from "./UpdatePing";

export default function UpdateButton({ enabled }: { enabled: boolean }) {
  async function handleUpdate() {
    try {
      const update = await check();
      if (update?.available) {
        await update?.downloadAndInstall();
        await relaunch();
      } else {
        toast.info("No update available.");
      }
    } catch (e) {
      toast.error(`Could not update!\n${JSON.stringify(e)}`);
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
