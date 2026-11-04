import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { useState } from "react";
import { toast } from "sonner";
import Button from "../layout/Button";
import UpdatePing from "./UpdatePing";

export default function UpdateButton({ enabled }: { enabled: boolean }) {
  const [updating, setUpdating] = useState(false);

  async function handleUpdate() {
    setUpdating(true);
    try {
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        await relaunch();
      } else {
        toast(`No update available`);
      }
    } catch (e) {
      toast.error(`Could not update!\n${JSON.stringify(e)}`);
    } finally {
      setUpdating(false);
    }
  }
  return (
    <UpdatePing canUpdate={enabled && !updating}>
      <Button
        disabled={!enabled || updating}
        onClick={handleUpdate}
        className="m-0"
      >
        {updating ? "Updating..." : "Update"}
      </Button>
    </UpdatePing>
  );
}
