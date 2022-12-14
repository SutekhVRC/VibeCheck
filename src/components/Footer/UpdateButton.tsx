import { installUpdate } from "@tauri-apps/api/updater";
import { relaunch } from "@tauri-apps/api/process";
import UpdatePing from "./UpdatePing";

export default function ({ enabled }: { enabled: boolean }) {
  async function handleUpdate() {
    try {
      await installUpdate();
      await relaunch();
    } catch (e) {
      console.log(e);
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
