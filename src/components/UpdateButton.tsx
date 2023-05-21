import { installUpdate } from "@tauri-apps/api/updater";
import { relaunch } from "@tauri-apps/api/process";
import UpdatePing from "./UpdatePing";
import { useToastContext } from "../context/ToastContext";

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
      <button
        disabled={!enabled}
        onClick={handleUpdate}
        className={`bg-zinc-100 rounded-md p-1 px-5 ${
          enabled && "text-zinc-900 hover:bg-zinc-200"
        }`}
      >
        Upgrade
      </button>
    </UpdatePing>
  );
}
