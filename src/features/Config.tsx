import type { FeVibeCheckConfig } from "@bindings/FeVibeCheckConfig";
import { invoke } from "@tauri-apps/api/core";
import { Settings } from "lucide-react";
import { FormEvent, ReactNode, useEffect, useState } from "react";
import { toast } from "sonner";
import EditableNumber from "../components/EditableNumber";
import PageHeader from "../components/PageHeader";
import UpdateButton from "../components/UpdateButton";
import { useCoreEventContext } from "../context/CoreEvents";
import { INVOKE, TOOLTIP } from "../data/constants";
import Button from "../layout/Button";
import Slider from "../layout/Slider";
import Switch from "../layout/Switch";
import { TooltipLabel } from "../layout/Tooltip";
import ThemePicker from "./ThemePicker";

const ADDRESS_PATTERN = String.raw`^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}:\d{1,5}$`;

/** How long the reset button waits for the confirming second click */
const CONFIRM_TIMEOUT_MS = 4000;

type GeneralKey =
  | "scan_on_disconnect"
  | "minimize_on_exit"
  | "desktop_notifications"
  | "show_toy_advanced"
  | "show_feature_advanced";

type GeneralSettings = Pick<FeVibeCheckConfig, GeneralKey>;

const GENERAL_TOGGLES: {
  key: GeneralKey;
  label: string;
  tooltip: (typeof TOOLTIP)[keyof typeof TOOLTIP];
}[] = [
  {
    key: "scan_on_disconnect",
    label: "Scan On Disconnect",
    tooltip: TOOLTIP.ScanOnDisconnect,
  },
  {
    key: "minimize_on_exit",
    label: "Minimize On Exit",
    tooltip: TOOLTIP.MinimizeOnExit,
  },
  {
    key: "desktop_notifications",
    label: "Desktop Notifications",
    tooltip: TOOLTIP.DesktopNotifications,
  },
  {
    key: "show_toy_advanced",
    label: "Open toy settings by default",
    tooltip: TOOLTIP.AdvancedToy,
  },
  {
    key: "show_feature_advanced",
    label: "Expand advanced feature options",
    tooltip: TOOLTIP.AdvancedFeature,
  },
];

function pickGeneral(config: FeVibeCheckConfig): GeneralSettings {
  return {
    scan_on_disconnect: config.scan_on_disconnect,
    minimize_on_exit: config.minimize_on_exit,
    desktop_notifications: config.desktop_notifications,
    show_toy_advanced: config.show_toy_advanced,
    show_feature_advanced: config.show_feature_advanced,
  };
}

function Section({
  title,
  hint,
  children,
}: {
  title: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <section className="rounded-2xl border border-theme-border/60 bg-theme-raised p-4 text-left text-sm">
      <div className="mb-3 flex items-baseline justify-between gap-2">
        <h2 className="text-base font-semibold text-theme-secondary">
          {title}
        </h2>
        {hint && <span className="text-xs text-theme-muted">{hint}</span>}
      </div>
      {children}
    </section>
  );
}

function AddressInput({
  name,
  value,
  onChange,
}: {
  name: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <input
      name={name}
      className="theme-input"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      pattern={ADDRESS_PATTERN}
      onInvalid={(e) =>
        (e.target as HTMLInputElement).setCustomValidity("Enter valid IP:PORT")
      }
      onInput={(e) => (e.target as HTMLInputElement).setCustomValidity("")}
    />
  );
}

export default function Config({
  config,
  refreshConfig,
  canUpdate,
  version,
  disableOnPortChange,
}: {
  config: FeVibeCheckConfig;
  refreshConfig: () => Promise<void>;
  canUpdate: boolean;
  version: string;
  disableOnPortChange: () => Promise<void>;
}) {
  const { autoEnableOnScan, setAutoEnableOnScan } = useCoreEventContext();

  // Toggles save instantly; kept locally so the switch flips immediately
  const [general, setGeneral] = useState(() => pickGeneral(config));

  // Kept locally while dragging; saved when the drag ends
  const [intensity, setIntensity] = useState(config.master_intensity);

  // Address edits are a draft until applied (applying turns the app off)
  const [draft, setDraft] = useState({
    bind: config.networking.bind,
    remote: config.networking.remote,
  });
  const connectionDirty =
    draft.bind !== config.networking.bind ||
    draft.remote !== config.networking.remote;

  const [confirmReset, setConfirmReset] = useState(false);
  useEffect(() => {
    if (!confirmReset) return;
    const t = setTimeout(() => setConfirmReset(false), CONFIRM_TIMEOUT_MS);
    return () => clearTimeout(t);
  }, [confirmReset]);

  async function saveToggle(key: GeneralKey, value: boolean) {
    const next = { ...general, [key]: value };
    setGeneral(next);
    try {
      // Always send the *saved* addresses, never a half-typed draft
      await invoke(INVOKE.SET_CONFIG, { feVcConfig: { ...config, ...next } });
      await refreshConfig();
    } catch (e) {
      setGeneral((g) => ({ ...g, [key]: !value }));
      toast.error(`Could not save setting!\n${JSON.stringify(e)}`);
    }
  }

  async function saveIntensity(value: number) {
    const previous = config.master_intensity;
    setIntensity(value);
    try {
      await invoke(INVOKE.SET_CONFIG, {
        feVcConfig: { ...config, ...general, master_intensity: value },
      });
      await refreshConfig();
    } catch (e) {
      setIntensity(previous);
      toast.error(`Could not save master intensity!\n${JSON.stringify(e)}`);
    }
  }

  async function applyConnection(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    try {
      await disableOnPortChange();
      await invoke(INVOKE.SET_CONFIG, {
        feVcConfig: {
          ...config,
          ...general,
          // Spread keeps fields we don't edit here (osc_query_enabled)
          networking: { ...config.networking, ...draft },
        },
      });
      await refreshConfig();
      toast.info("Connection settings applied");
    } catch (e) {
      toast.error(`Could not save connection settings!\n${JSON.stringify(e)}`);
    }
  }

  async function resetOsc() {
    if (!confirmReset) {
      setConfirmReset(true);
      return;
    }
    setConfirmReset(false);
    try {
      await invoke(INVOKE.CLEAR_OSC_CONFIG);
      toast.info(
        "Cleared avatar OSC configs\nRemoved AppData\\LocalLow\\VRChat\\VRChat\\OSC",
      );
    } catch (e) {
      toast.error(`Could not clear avatar OSC configs!\n${JSON.stringify(e)}`);
    }
  }

  return (
    <div className="flex h-full w-full flex-col">
      {/* Same insets as the toy page, so switching pages doesn't shift things */}
      <PageHeader
        className="px-2 pb-2"
        icon={<Settings className="h-5 w-5" />}
        title="Settings"
        subtitle="Connection, behaviour and appearance"
      />

      <div className="scrollbar min-h-0 w-full flex-1 overflow-y-auto">
        <div className="flex flex-col gap-4 px-2 pb-4 pt-3">
          <Section title="Connection">
            <form onSubmit={applyConnection}>
              <div className="grid grid-cols-[minmax(8rem,1fr)_minmax(8rem,2fr)] items-center gap-x-4 gap-y-2">
                <TooltipLabel text="OSC Bind" tooltip={TOOLTIP.OSC_Bind} />
                <AddressInput
                  name="bind"
                  value={draft.bind}
                  onChange={(bind) => setDraft((d) => ({ ...d, bind }))}
                />
                <TooltipLabel text="OSC Remote" tooltip={TOOLTIP.OSC_Remote} />
                <AddressInput
                  name="remote"
                  value={draft.remote}
                  onChange={(remote) => setDraft((d) => ({ ...d, remote }))}
                />
              </div>
              {connectionDirty && (
                <div className="mt-3 flex flex-wrap items-center justify-end gap-2">
                  <span className="mr-auto text-xs text-theme-muted">
                    Applying turns VibeCheck off. Turn it back on afterwards.
                  </span>
                  <Button
                    type="button"
                    variant="secondary"
                    className="m-0"
                    onClick={() =>
                      setDraft({
                        bind: config.networking.bind,
                        remote: config.networking.remote,
                      })
                    }
                  >
                    Revert
                  </Button>
                  <Button type="submit" className="m-0">
                    Apply
                  </Button>
                </div>
              )}
            </form>
          </Section>

          <Section title="Toys" hint="Saves instantly">
            <div className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-x-4">
              <TooltipLabel
                text="Master intensity"
                tooltip={TOOLTIP.MasterIntensity}
              />
              <Slider
                multiply={100}
                min={0}
                max={1}
                step={0.01}
                value={[intensity]}
                onValueChange={(v) => setIntensity(v[0])}
                onValueCommit={(v) => saveIntensity(v[0])}
              />
              <EditableNumber
                value={intensity}
                min={0}
                max={1}
                step={0.01}
                scale={100}
                onCommit={saveIntensity}
                label="Master intensity"
              />
            </div>
          </Section>

          <Section title="General" hint="Saves instantly">
            <div className="grid grid-cols-[1fr_auto] items-center gap-y-3">
              {GENERAL_TOGGLES.map(({ key, label, tooltip }) => (
                <div key={key} className="contents">
                  <TooltipLabel text={label} tooltip={tooltip} />
                  <Switch
                    size="small"
                    checked={general[key]}
                    onCheckedChange={(checked) => saveToggle(key, checked)}
                  />
                </div>
              ))}
              {/* Frontend-only preference, saved on this device */}
              <TooltipLabel
                text="Turn on when scanning"
                tooltip={TOOLTIP.AutoEnableOnScan}
              />
              <Switch
                size="small"
                checked={autoEnableOnScan}
                onCheckedChange={setAutoEnableOnScan}
              />
            </div>
          </Section>

          <Section title="Theme" hint="Saves instantly">
            <ThemePicker />
          </Section>

          <Section title="Troubleshooting">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="min-w-[12rem] flex-1">
                <div className="text-theme-text">Reset VRChat OSC configs</div>
                <div className="text-xs text-theme-muted">
                  Deletes VRChat&apos;s saved avatar OSC files so they get
                  regenerated. Use this if avatar parameters stop working.
                </div>
              </div>
              <Button variant="danger" className="m-0" onClick={resetOsc}>
                {confirmReset ? "Click again to confirm" : "Reset"}
              </Button>
            </div>
          </Section>

          <Section title="About">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <div className="text-theme-text">
                  VibeCheck {version || "…"}
                </div>
                <div
                  className={
                    canUpdate
                      ? "text-xs text-theme-success"
                      : "text-xs text-theme-muted"
                  }
                >
                  {canUpdate ? "Update available" : "Up to date"}
                </div>
              </div>
              {canUpdate && <UpdateButton enabled={canUpdate} />}
            </div>
          </Section>
        </div>
      </div>
    </div>
  );
}
