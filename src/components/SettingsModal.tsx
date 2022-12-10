import { useRef, useState } from "react";
import { invoke } from "@tauri-apps/api";

import type { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import { SET_CONFIG } from "../data/constants";
import { Dialog } from "@headlessui/react";
import { useCoreEventContext } from "../context/CoreEventContext";
import Modal from "./Modal";

type settingsDialogProps = {
  open: boolean;
  onClose: () => void;
};

export default function ({ open: isOpen, onClose }: settingsDialogProps) {
  // TODO maybe go back to ref
  // Is react complaining that I use the context to set default state?
  const { config, refreshConfig } = useCoreEventContext();
  if (config == null) {
    return (
      <Dialog open={isOpen} onClose={onClose}>
        <Dialog.Panel>Could not load settings</Dialog.Panel>;
      </Dialog>
    );
  }

  const oscBind = useRef<HTMLInputElement>(null);

  const [scanOnDisconnect, setScanOnDisconnect] = useState(
    config.scan_on_disconnect
  );
  const [minimizeOnExit, setMinimizeOnExit] = useState(config.minimize_on_exit);
  const [desktopNotifications, setDesktopNotifications] = useState(
    config.desktop_notifications
  );

  async function saveConfig() {
    if (config == null || !oscBind.current) {
      return;
    }
    const newConfig: FeVibeCheckConfig = {
      ...config,
      scan_on_disconnect: scanOnDisconnect,
      minimize_on_exit: minimizeOnExit,
      desktop_notifications: desktopNotifications,
      networking: {
        ...config.networking,
        bind: oscBind.current.value,
      },
    };
    if (JSON.stringify(config) == JSON.stringify(newConfig)) return;
    await invoke(SET_CONFIG, { feVcConfig: newConfig });
    refreshConfig();
  }

  return (
    <Modal title="Settings" isOpen={isOpen} onClose={onClose}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          saveConfig();
          onClose();
        }}
      >
        <div className="grid grid-cols-2 gap-y-2 justify-items-start">
          <label>OSC Bind</label>
          <input
            className="text-zinc-800"
            defaultValue={config.networking.bind}
            ref={oscBind}
            pattern={String.raw`^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}:\d{1,5}$`}
            onInvalid={(e) =>
              (e.target as HTMLInputElement).setCustomValidity(
                "Enter valid IP:HOST"
              )
            }
            onInput={(e) =>
              (e.target as HTMLInputElement).setCustomValidity("")
            }
          />
          <label>Scan on Disconnect</label>
          <input
            type="checkbox"
            checked={scanOnDisconnect}
            onChange={() => setScanOnDisconnect((e) => !e)}
          />
          <label>Minimize on Exit</label>
          <input
            type="checkbox"
            checked={minimizeOnExit}
            onChange={() => setMinimizeOnExit((e) => !e)}
          />
          <label>Desktop Notifications</label>
          <input
            type="checkbox"
            checked={desktopNotifications}
            onChange={() => setDesktopNotifications((e) => !e)}
          />
        </div>
        <div className="mt-4">
          <button
            type="submit"
            className="inline-flex justify-center rounded-md bg-zinc-100 px-4 py-2  text-zinc-900 hover:bg-zinc-200"
          >
            Save
          </button>
        </div>
      </form>
    </Modal>
  );
}
