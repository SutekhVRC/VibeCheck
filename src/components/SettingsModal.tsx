import type { ChangeEvent } from "react";
import { useEffect } from "react";
import { useState } from "react";
import { invoke } from "@tauri-apps/api";

import type { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import { SET_CONFIG } from "../data/constants";
import Modal from "./Modal";

type settingsDialogProps = {
  isOpen: boolean;
  onClose: () => void;
  config: FeVibeCheckConfig;
};

export default function ({ isOpen, onClose, config }: settingsDialogProps) {
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

  useEffect(() => {
    // onOpen refresh to backend config
    if (!isOpen) return;
    setNewConfig(config);
  }, [isOpen]);

  return (
    <Modal title="Settings" isOpen={isOpen} onClose={onClose}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          saveConfig();
          onClose();
        }}
      >
        <div className="grid grid-cols-2 gap-y-2 justify-items-end">
          <label className="justify-self-start">OSC Bind</label>
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
          <label className="justify-self-start">OSC Remote</label>
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
          <label className="justify-self-start">Lovense Connect Override</label>
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
          <label className="justify-self-start">Scan on Disconnect</label>
          <input
            name="scan_on_disconnect"
            type="checkbox"
            checked={newConfig.scan_on_disconnect}
            onChange={onCheck}
          />
          <label className="justify-self-start">Minimize on Exit</label>
          <input
            name="minimize_on_exit"
            type="checkbox"
            checked={newConfig.minimize_on_exit}
            onChange={onCheck}
          />
          <label className="justify-self-start">Desktop Notifications</label>
          <input
            name="desktop_notifications"
            type="checkbox"
            checked={newConfig.desktop_notifications}
            onChange={onCheck}
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
