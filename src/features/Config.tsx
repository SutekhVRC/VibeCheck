import type { ChangeEvent, FormEvent } from "react";
import { useEffect } from "react";
import { useState } from "react";
import { invoke } from "@tauri-apps/api";

import type { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import { SET_CONFIG } from "../data/constants";
import Modal from "../layout/Modal";
import UpdateButton from "../components/UpdateButton";
import TooltipLabel from "../layout/Tooltip/TooltipLabel";

type settingsDialogProps = {
  isOpen: boolean;
  onClose: () => void;
  config: FeVibeCheckConfig;
  canUpdate: boolean;
};

export default function Config({
  isOpen,
  onClose,
  config,
  canUpdate,
}: settingsDialogProps) {
  const [newConfig, setNewConfig] = useState<FeVibeCheckConfig>(config);

  const onChange = (e: ChangeEvent<HTMLInputElement>) => {
    setNewConfig({ ...newConfig, [e.target.name]: e.target.value });
  };

  const onCheck = (e: ChangeEvent<HTMLInputElement>) => {
    setNewConfig({ ...newConfig, [e.target.name]: e.target.checked });
  };

  const onChangeNetworking = (e: ChangeEvent<HTMLInputElement>) => {
    setNewConfig({
      ...newConfig,
      networking: {
        ...newConfig.networking,
        [e.target.name]: e.target.value,
      },
    });
  };

  async function saveConfig() {
    await invoke(SET_CONFIG, { feVcConfig: newConfig });
  }

  function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    saveConfig();
    onClose();
  }

  useEffect(() => {
    // onOpen refresh to backend config
    if (!isOpen) return;
    setNewConfig(config);
  }, [isOpen]);

  return (
    <Modal title="Config" isOpen={isOpen} onClose={onClose}>
      <form id="config" onSubmit={handleSubmit}>
        <div className="grid grid-cols-2 gap-y-2 justify-items-end">
          <TooltipLabel text="OSC Bind" tooltip="OSC Receive Port" />
          <input
            name="bind"
            className="text-zinc-800"
            value={newConfig.networking.bind}
            onChange={onChangeNetworking}
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
          <TooltipLabel text="OSC Remote" tooltip="OSC Send Port" />
          <input
            name="remote"
            className="text-zinc-800"
            value={newConfig.networking.remote}
            onChange={onChangeNetworking}
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
          <TooltipLabel
            text="Lovense Connect Override"
            tooltip="Override and force the Lovense Connect host to connect to."
          />
          <div className="">
            <input
              type="checkbox"
              checked={newConfig.lc_override != null}
              onChange={(e) =>
                setNewConfig((v) => {
                  return {
                    ...v,
                    lc_override: e.target.checked ? "127.0.0.1" : null,
                  };
                })
              }
            />
            {newConfig.lc_override != null && (
              <input
                name="lc_override"
                className="text-zinc-800 ml-2"
                value={newConfig.lc_override}
                onChange={onChange}
                pattern={String.raw`^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$`}
                onInvalid={(e) =>
                  (e.target as HTMLInputElement).setCustomValidity(
                    "Enter valid IP"
                  )
                }
                onInput={(e) =>
                  (e.target as HTMLInputElement).setCustomValidity("")
                }
              />
            )}
          </div>
          <TooltipLabel
            text="Scan On Disconnect"
            tooltip="Automatically start scanning when a toy disconnects."
          />
          <input
            name="scan_on_disconnect"
            type="checkbox"
            checked={newConfig.scan_on_disconnect}
            onChange={onCheck}
          />
          <TooltipLabel
            text="Minimize On Exit"
            tooltip="Minimize VibeCheck instead of exiting."
          />
          <input
            name="minimize_on_exit"
            type="checkbox"
            checked={newConfig.minimize_on_exit}
            onChange={onCheck}
          />
          <TooltipLabel
            text="Desktop Notifications"
            tooltip="Notifications for toy connect and disconnect."
          />
          <input
            name="desktop_notifications"
            type="checkbox"
            checked={newConfig.desktop_notifications}
            onChange={onCheck}
          />
        </div>
      </form>
      <div className="mt-4 flex justify-around">
        <button
          type="submit"
          form="config"
          className="rounded-md bg-zinc-100 px-4 text-zinc-900 hover:bg-zinc-200"
        >
          Save
        </button>
        <UpdateButton enabled={canUpdate} />
      </div>
    </Modal>
  );
}
