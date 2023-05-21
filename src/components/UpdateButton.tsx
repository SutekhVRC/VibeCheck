import { installUpdate } from "@tauri-apps/api/updater";
import { relaunch } from "@tauri-apps/api/process";
import UpdatePing from "./UpdatePing";
import { useToastContext } from "../context/ToastContext";
import Button from "../layout/Button";

export default function UpdateButton({ enabled }: { enabled: boolean }) {
  const toast = useToastContext();

  async function handleUpdate() {
    try {
      await installUpdate();
      await relaunch();
    } catch (e) {
      toast.createToast("Could not update!", `${e}`, "error");
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
