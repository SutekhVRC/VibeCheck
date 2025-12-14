import type { FeVibeCheckConfig } from "@bindings/FeVibeCheckConfig";
import { invoke } from "@tauri-apps/api/core";
import { ChangeEvent, FormEvent, useState } from "react";
import { toast } from "sonner";
import UpdateButton from "../components/UpdateButton";
import { INVOKE, TOOLTIP } from "../data/constants";
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

  const onCheckSwitch = (checked: boolean, name: keyof FeVibeCheckConfig) => {
    setNewConfig({ ...newConfig, [name]: checked });
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
      toast.info("Saved config");
    } catch (e) {
      toast.error(`Could not save config!\n${JSON.stringify(e)}`);
    }
  }

  async function clearOsc() {
    try {
      await invoke(INVOKE.CLEAR_OSC_CONFIG);
      toast.info(
        "Cleared avatar OSC configs\nRemoved AppData\\LocalLow\\VRChat\\VRChat\\OSC",
      );
    } catch (e) {
      toast.error(`Could not clear avatar OSC configs!\n${JSON.stringify(e)}`);
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
            <TooltipLabel text="OSC Bind" tooltip={TOOLTIP.OSC_Bind} />
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
            <TooltipLabel text="OSC Remote" tooltip={TOOLTIP.OSC_Remote} />
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
              text="Scan On Disconnect"
              tooltip={TOOLTIP.ScanOnDisconnect}
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
              tooltip={TOOLTIP.MinimizeOnExit}
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
              tooltip={TOOLTIP.DesktopNotifications}
            />
            <Switch
              checked={newConfig.desktop_notifications}
              onCheckedChange={(checked) =>
                onCheckSwitch(checked, "desktop_notifications")
              }
              size="small"
            />
            <div />
            <TooltipLabel
              text="Advanced toy options"
              tooltip={TOOLTIP.AdvancedToy}
            />
            <Switch
              checked={newConfig.show_toy_advanced}
              onCheckedChange={(checked) =>
                onCheckSwitch(checked, "show_toy_advanced")
              }
              size="small"
            />
            <div />
            <TooltipLabel
              text="Advanced feature options"
              tooltip={TOOLTIP.AdvancedFeature}
            />
            <Switch
              checked={newConfig.show_feature_advanced}
              onCheckedChange={(checked) =>
                onCheckSwitch(checked, "show_feature_advanced")
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
