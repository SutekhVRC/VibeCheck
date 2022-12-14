import { installUpdate } from "@tauri-apps/api/updater";
import { relaunch } from "@tauri-apps/api/process";
import UpdatePing from "./UpdatePing";

export default function () {
  async function handleUpdate() {
    try {
      await installUpdate();
      await relaunch();
    } catch (e) {
      console.log(e);
    }
  }
  return (
    <div className="flex justify-center">
      <UpdatePing canUpdate={true}>
        <button
          onClick={handleUpdate}
          className="bg-gray-700 border-solid border-gray-400 border-2 rounded-md p-1 pl-5 pr-5 m-2 hover:bg-gray-800"
        >
          Upgrade
        </button>
      </UpdatePing>
    </div>
  );
}
