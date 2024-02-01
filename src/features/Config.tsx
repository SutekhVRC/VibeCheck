import { invoke } from "@tauri-apps/api";
import { ChangeEvent, FormEvent, useState } from "react";
import type { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import { createToast } from "../components/Toast";
import UpdateButton from "../components/UpdateButton";
import { INVOKE } from "../data/constants";
import Button from "../layout/Button";
import Switch from "../layout/Switch";
import { TooltipLabel } from "../layout/Tooltip";

export default function Config({
  config,
  refreshConfig,
  canUpdate,
  disableOnPortChange,
}: {
  config: FeVibeCheckConfig;
  refreshConfig: () => Promise<void>;
  canUpdate: boolean;
  disableOnPortChange: () => Promise<void>;
}) {
  const [newConfig, setNewConfig] = useState<FeVibeCheckConfig>(config);

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
      if (
        newConfig.networking.bind != config.networking.bind ||
        newConfig.networking.remote != config.networking.remote
      ) {
        await disableOnPortChange();
      }
      await invoke(INVOKE.SET_CONFIG, { feVcConfig: newConfig });
      createToast("info", "Saved config");
    } catch (e) {
      createToast("error", "Could not set config!", JSON.stringify(e));
    }
  }

  async function clearOsc() {
    try {
      await invoke(INVOKE.CLEAR_OSC_CONFIG);
      createToast(
        "info",
        "Cleared avatar OSC configs",
        "Removed AppData\\LocalLow\\VRChat\\VRChat\\OSC",
      );
    } catch (e) {
      createToast(
        "error",
        "Could not clear avatar OSC configs!",
        JSON.stringify(e),
      );
    }
  }

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    await saveConfig();
    await refreshConfig();
  }

  return (
    <div className="h-full w-full">
      <div className="flex items-center justify-between px-6 text-4xl">
        <div className="flex items-end gap-2">Config</div>
      </div>
      <div className="flex w-full flex-col justify-between">
        <form id="config" onSubmit={handleSubmit}>
          <div className="mx-8 my-4 grid grid-cols-[minmax(10rem,4fr)_1fr_minmax(4rem,_4fr)] gap-1 text-justify text-sm">
            <TooltipLabel
              text="OSC Bind"
              tooltip="OSC Receive Port (Default: 127.0.0.1:9001)"
            />
            <div />
            <input
              name="bind"
              className="rounded-sm px-1 text-zinc-800 outline-none"
              value={newConfig.networking.bind}
              onChange={onChangeNetworking}
              pattern={String.raw`^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}:\d{1,5}$`}
              onInvalid={(e) =>
                (e.target as HTMLInputElement).setCustomValidity(
                  "Enter valid IP:HOST",
                )
              }
              onInput={(e) =>
                (e.target as HTMLInputElement).setCustomValidity("")
              }
            />
            <TooltipLabel
              text="OSC Remote"
              tooltip="OSC Send Port (Default: 127.0.0.1:9000)"
            />
            <div />
            <input
              name="remote"
              className="rounded-sm px-1 text-zinc-800 outline-none"
              value={newConfig.networking.remote}
              onChange={onChangeNetworking}
              pattern={String.raw`^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}:\d{1,5}$`}
              onInvalid={(e) =>
                (e.target as HTMLInputElement).setCustomValidity(
                  "Enter valid IP:HOST",
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
              onCheckedChange={handleLcOverride}
              size="small"
            />
            {newConfig.lc_override == null ? (
              <div />
            ) : (
              <input
                name="lc_override"
                className="px-1 text-zinc-800"
                value={newConfig.lc_override}
                onChange={onChange}
                pattern={String.raw`^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$`}
                onInvalid={(e) =>
                  (e.target as HTMLInputElement).setCustomValidity(
                    "Enter valid IP",
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
              onCheckedChange={(checked) =>
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
              onCheckedChange={(checked) =>
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
              onCheckedChange={(checked) =>
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
