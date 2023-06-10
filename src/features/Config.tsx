import { ChangeEvent, FormEvent } from "react";
import { useState } from "react";
import { invoke } from "@tauri-apps/api";
import type { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import { CLEAR_OSC_CONFIG, SET_CONFIG } from "../data/constants";
import UpdateButton from "../components/UpdateButton";
import { TooltipLabel } from "../layout/Tooltip";
import Switch from "../layout/Switch";
import Button from "../layout/Button";
import { useUpdate } from "../hooks/useUpdate";
import { useCoreEventContext } from "../context/CoreEventContext";
import { createToast } from "../components/Toast";

export default function Config() {
  const { config, refreshConfig } = useCoreEventContext();
  if (config == null) return null;
  const [newConfig, setNewConfig] = useState<FeVibeCheckConfig>(config);
  const { canUpdate } = useUpdate();

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
      createToast("info", "Saved config");
    } catch (e) {
      createToast("error", "Could not set config!", JSON.stringify(e));
    }
  }

  async function clearOsc() {
    try {
      await invoke(CLEAR_OSC_CONFIG);
      createToast(
        "info",
        "Cleared avatar OSC configs",
        "Removed AppData\\LocalLow\\VRChat\\VRChat\\OSC"
      );
    } catch (e) {
      createToast(
        "error",
        "Could not clear avatar OSC configs!",
        JSON.stringify(e)
      );
    }
  }

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    await saveConfig();
    refreshConfig();
  }

  return (
    <div className="w-full h-full pb-6">
      <div className="text-4xl flex justify-between items-center px-6">
        <div className="flex items-end gap-2">Config</div>
      </div>
      <div className="flex flex-col justify-between h-full">
        <form id="config" onSubmit={handleSubmit}>
          <div className="grid grid-cols-[minmax(10rem,4fr)_1fr_minmax(6rem,_4fr)] text-sm text-justify gap-y-1 gap-x-2 my-4 mx-8">
            <TooltipLabel text="OSC Bind" tooltip="OSC Receive Port" />
            <div />
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
            <TooltipLabel text="OSC Remote" tooltip="OSC Send Port" />
            <div />
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
            <TooltipLabel
              text="Lovense Connect Override"
              tooltip="Override and force the Lovense Connect host to connect to."
            />
            <Switch
              checked={newConfig.lc_override != null}
              onChange={handleLcOverride}
              size="small"
            />
            {newConfig.lc_override == null ? (
              <div />
            ) : (
              <input
                name="lc_override"
                className="text-zinc-800 px-1"
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
            <TooltipLabel
              text="Scan On Disconnect"
              tooltip="Automatically start scanning when a toy disconnects."
            />
            <Switch
              checked={newConfig.scan_on_disconnect}
              onChange={(checked: boolean) =>
                onCheckSwitch(checked, "scan_on_disconnect")
              }
              size="small"
            />
            <div />
            <TooltipLabel
              text="Minimize On Exit"
              tooltip="Minimize VibeCheck instead of exiting."
            />
            <Switch
              checked={newConfig.minimize_on_exit}
              onChange={(checked: boolean) =>
                onCheckSwitch(checked, "minimize_on_exit")
              }
              size="small"
            />
            <div />
            <TooltipLabel
              text="Desktop Notifications"
              tooltip="Notifications for toy connect and disconnect."
            />
            <Switch
              checked={newConfig.desktop_notifications}
              onChange={(checked: boolean) =>
                onCheckSwitch(checked, "desktop_notifications")
              }
              size="small"
            />
            <div />
          </div>
        </form>
        <div className="flex justify-around">
          <Button type="submit" form="config">
            Save
          </Button>
          <Button onClick={clearOsc}>Refresh OSC</Button>
          <UpdateButton enabled={canUpdate} />
        </div>
      </div>
    </div>
  );
}
