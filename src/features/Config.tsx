import type { ChangeEvent, FormEvent } from "react";
import { useEffect } from "react";
import { useState } from "react";
import { invoke } from "@tauri-apps/api";

import type { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import { CLEAR_OSC_CONFIG, SET_CONFIG } from "../data/constants";
import Modal from "../layout/Modal";
import UpdateButton from "../components/UpdateButton";
import { TooltipLabel } from "../layout/Tooltip";
import Tooltip from "../layout/Tooltip";
import Switch from "../layout/Switch";
import { useToastContext } from "../context/ToastContext";
import Button from "../layout/Button";

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
  const [refreshDisabled, setRefreshDisabled] = useState(false);
  const toast = useToastContext();

  const onChange = (e: ChangeEvent<HTMLInputElement>) => {
    setNewConfig({ ...newConfig, [e.target.name]: e.target.value });
  };

  const onCheckSwitch = (checked: boolean, name: keyof FeVibeCheckConfig) => {
    setNewConfig({ ...newConfig, [name]: checked });
  };

  const handleLcOverride = () => {
    if (newConfig.lc_override == null)
      setNewConfig({ ...newConfig, lc_override: "127.0.0.1" });
    else setNewConfig({ ...newConfig, lc_override: null });
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
    try {
      await invoke(SET_CONFIG, { feVcConfig: newConfig });
      setRefreshDisabled(false);
    } catch (e) {
      toast.createToast("Could not set config!", JSON.stringify(e), "error");
    }
  }

  async function refreshConfig() {
    try {
      await invoke(CLEAR_OSC_CONFIG);
      setRefreshDisabled(true);
    } catch (e) {
      toast.createToast(
        "Could not clear avatar OSC configs!",
        JSON.stringify(e),
        "error"
      );
    }
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
    <Modal title="Vibecheck Config" isOpen={isOpen} onClose={onClose}>
      <form id="config" onSubmit={handleSubmit}>
        <div className="grid grid-cols-[minmax(0,_6fr)_minmax(0,_4fr)_minmax(0,_1fr)] gap-3 my-4 items-center">
          <TooltipLabel text="OSC Bind" tooltip="OSC Receive Port" />
          <input
            name="bind"
            className="text-zinc-800 px-1 rounded-sm outline-none"
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
          <div />
          <TooltipLabel text="OSC Remote" tooltip="OSC Send Port" />
          <input
            name="remote"
            className="text-zinc-800 px-1 rounded-sm outline-none"
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
          <div />
          <TooltipLabel
            text="Lovense Connect Override"
            tooltip="Override and force the Lovense Connect host to connect to."
          />
          {newConfig.lc_override == null ? (
            <div />
          ) : (
            <input
              name="lc_override"
              className="text-zinc-800"
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
          <Switch
            checked={newConfig.lc_override != null}
            onChange={handleLcOverride}
            size="small"
          />
          <TooltipLabel
            text="Scan On Disconnect"
            tooltip="Automatically start scanning when a toy disconnects."
          />
          <div />
          <Switch
            checked={newConfig.scan_on_disconnect}
            onChange={(checked: boolean) =>
              onCheckSwitch(checked, "scan_on_disconnect")
            }
            size="small"
          />
          <TooltipLabel
            text="Minimize On Exit"
            tooltip="Minimize VibeCheck instead of exiting."
          />
          <div />
          <Switch
            checked={newConfig.minimize_on_exit}
            onChange={(checked: boolean) =>
              onCheckSwitch(checked, "minimize_on_exit")
            }
            size="small"
          />
          <TooltipLabel
            text="Desktop Notifications"
            tooltip="Notifications for toy connect and disconnect."
          />
          <div />
          <Switch
            checked={newConfig.desktop_notifications}
            onChange={(checked: boolean) =>
              onCheckSwitch(checked, "desktop_notifications")
            }
            size="small"
          />
        </div>
      </form>
      <div className="flex justify-around">
        <Button type="submit" form="config">
          Save
        </Button>
        <Tooltip text="Force refresh OSC avatar parameters by deleting VRChat OSC config folders. The in-game button does not work.">
          <Button disabled={refreshDisabled} onClick={refreshConfig}>
            Refresh OSC
          </Button>
        </Tooltip>
        <UpdateButton enabled={canUpdate} />
      </div>
    </Modal>
  );
}
