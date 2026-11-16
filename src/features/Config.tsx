import { errorCode, errorText } from "@/lib/errors";
import type { FeOscMode } from "@bindings/FeOscMode";
import type { FeOscQueryStatus } from "@bindings/FeOscQueryStatus";
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
import { appExtensions } from "../extensions";
import type { UpdateStatus } from "../hooks/useUpdate";
import Button from "../layout/Button";
import { Select } from "../layout/Select";
import Slider from "../layout/Slider";
import Switch from "../layout/Switch";
import { TooltipLabel } from "../layout/Tooltip";
import ThemePicker from "./ThemePicker";

const UPDATE_TEXT: Record<UpdateStatus, string> = {
  checking: "Checking for updates…",
  available: "Update available",
  current: "Up to date",
  failed: "Couldn't check for updates",
  off: "",
};

const ADDRESS_PATTERN = String.raw`^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}:\d{1,5}$`;

/** Scan length choices, in seconds (0: until stopped) */
const SCAN_LENGTHS: [number, string][] = [
  [10, "10 seconds"],
  [30, "30 seconds"],
  [60, "1 minute"],
  [120, "2 minutes"],
  [300, "5 minutes"],
  [0, "Until I stop it"],
];

function scanLengthLabel(seconds: number) {
  return SCAN_LENGTHS.find(([s]) => s === seconds)?.[1] ?? `${seconds} seconds`;
}

/** How often the OSCQuery status refreshes while Settings is open */
const OSCQUERY_STATUS_MS = 3000;

/** How long the reset buttons wait for the confirming second click */
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

/** Polls OSCQuery mode's status while `enabled` */
function useOscQueryStatus(enabled: boolean) {
  const [status, setStatus] = useState<FeOscQueryStatus | null>(null);
  useEffect(() => {
    if (!enabled) {
      setStatus(null);
      return;
    }
    let cancelled = false;
    async function refresh() {
      try {
        const s = await invoke<FeOscQueryStatus>(INVOKE.GET_OSCQUERY_STATUS);
        if (!cancelled) setStatus(s);
      } catch {
        // Keep the last status; it refreshes again shortly
      }
    }
    refresh();
    const timer = setInterval(refresh, OSCQUERY_STATUS_MS);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [enabled]);
  return status;
}

function ModeButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={active}
      className={
        active
          ? "rounded-full bg-theme-primary/20 px-3 py-1 text-sm font-semibold text-theme-primary ring-1 ring-theme-primary/60"
          : "rounded-full px-3 py-1 text-sm text-theme-muted transition-colors hover:text-theme-text"
      }
    >
      {children}
    </button>
  );
}

/** What automatic mode is doing: VibeCheck's port, and whether VRChat was found */
function OscQueryPanel({
  status,
  port,
}: {
  status: FeOscQueryStatus | null;
  port: number | null;
}) {
  const listening = status?.error
    ? `Couldn't advertise VibeCheck: ${status.error}`
    : status?.advertising
      ? `Port ${status.advertising}, on all network interfaces`
      : port
        ? `Port ${port} (starting…)`
        : "Starting…";
  const vrchat = status?.vrchat
    ? `Found. Battery levels go to ${status.vrchat}`
    : "Not found yet. Is VRChat running with OSC on?";
  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-[minmax(8rem,1fr)_minmax(8rem,2fr)] items-center gap-x-4 gap-y-2">
        <span className="text-theme-text">VibeCheck listens on</span>
        <span
          className={status?.error ? "text-theme-danger" : "text-theme-text"}
        >
          {listening}
        </span>
        <span className="text-theme-text">Port 9001</span>
        <span className="text-theme-muted">
          {!status?.advertising
            ? "…"
            : status.holding_default_port
              ? "Held by VibeCheck, so VRChat's default sends don't fail"
              : "In use by another app"}
        </span>
        <span className="text-theme-text">VRChat</span>
        <span
          className={status?.vrchat ? "text-theme-success" : "text-theme-muted"}
        >
          {vrchat}
        </span>
      </div>
      <p className="text-xs text-theme-muted">
        VRChat finds VibeCheck on its own and sends to this port, so it
        doesn&rsquo;t clash with other OSC apps. The port is reachable from your
        local network. If VRChat doesn&rsquo;t send anything, turn OSC off and
        on again in VRChat&rsquo;s Action Menu, or use Manual.
      </p>
    </div>
  );
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
  updateStatus,
  version,
  appName,
  disableOnPortChange,
}: {
  config: FeVibeCheckConfig;
  refreshConfig: () => Promise<void>;
  canUpdate: boolean;
  updateStatus: UpdateStatus;
  version: string;
  /** The app's product name, e.g. VibeCheck */
  appName: string;
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

  const [confirmSettingsReset, setConfirmSettingsReset] = useState(false);
  useEffect(() => {
    if (!confirmSettingsReset) return;
    const t = setTimeout(
      () => setConfirmSettingsReset(false),
      CONFIRM_TIMEOUT_MS,
    );
    return () => clearTimeout(t);
  }, [confirmSettingsReset]);

  async function saveToggle(key: GeneralKey, value: boolean) {
    const next = { ...general, [key]: value };
    setGeneral(next);
    try {
      // Always send the *saved* addresses, never a half-typed draft
      await invoke(INVOKE.SET_CONFIG, { feVcConfig: { ...config, ...next } });
      await refreshConfig();
    } catch (e) {
      setGeneral((g) => ({ ...g, [key]: !value }));
      toast.error(`Could not save setting!\n${errorText(e)}`);
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
      toast.error(`Could not save master intensity!\n${errorText(e)}`);
    }
  }

  async function saveScanLength(label: string) {
    const seconds = SCAN_LENGTHS.find(([, l]) => l === label)?.[0];
    if (seconds === undefined) return;
    try {
      await invoke(INVOKE.SET_CONFIG, {
        feVcConfig: { ...config, ...general, scan_length: seconds },
      });
      await refreshConfig();
    } catch (e) {
      toast.error(`Could not save scan length!\n${errorText(e)}`);
    }
  }

  // A length set by hand in Config.json still shows up as a choice
  const scanLengthOptions = [
    ...new Set([
      ...SCAN_LENGTHS.map(([, label]) => label),
      scanLengthLabel(config.scan_length),
    ]),
  ];

  const mode = config.networking.mode;
  const oscQueryStatus = useOscQueryStatus(mode === "OscQuery");

  // Switching mode applies straight away (it turns VibeCheck off, like
  // changing the addresses does)
  async function switchMode(next: FeOscMode) {
    if (next === mode) return;
    try {
      await disableOnPortChange();
      await invoke(INVOKE.SET_CONFIG, {
        feVcConfig: {
          ...config,
          ...general,
          networking: { ...config.networking, mode: next },
        },
      });
      await refreshConfig();
      toast.info(
        next === "OscQuery"
          ? "Switched to automatic (OSCQuery). Turn VibeCheck back on."
          : "Switched to manual addresses. Turn VibeCheck back on.",
      );
    } catch (e) {
      toast.error(`Could not switch connection mode!\n${errorText(e)}`);
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
          // Spread keeps fields we don't edit here (mode, oscquery_port)
          networking: { ...config.networking, ...draft },
        },
      });
      await refreshConfig();
      toast.info("Connection settings applied");
    } catch (e) {
      toast.error(`Could not save connection settings!\n${errorText(e)}`);
    }
  }

  async function resetOsc() {
    if (!confirmReset) {
      setConfirmReset(true);
      return;
    }
    setConfirmReset(false);
    try {
      const cleared = await invoke<string[]>(INVOKE.CLEAR_OSC_CONFIG);
      toast.info(
        `Cleared avatar OSC configs. Reload your avatar in VRChat.\n${cleared.join("\n")}`,
      );
    } catch (e) {
      if (errorCode(e) === "VRChatOscNotFound") {
        toast.error(
          "Couldn't find VRChat's OSC folder\nIn VRChat, use Action Menu > Options > OSC > Reset Config instead",
        );
      } else {
        toast.error(`Could not clear avatar OSC configs!\n${errorText(e)}`);
      }
    }
  }

  // Every app setting back to its default; toy settings stay. Turns
  // VibeCheck off first, since the connection settings reset too.
  async function openLogFolder() {
    try {
      await invoke(INVOKE.OPEN_LOG_FOLDER);
    } catch (e) {
      toast.error(`Could not open the log folder\n${errorText(e)}`);
    }
  }

  async function resetSettings() {
    if (!confirmSettingsReset) {
      setConfirmSettingsReset(true);
      return;
    }
    setConfirmSettingsReset(false);
    try {
      await disableOnPortChange();
      await invoke(INVOKE.RESET_CONFIG);
      const fresh = await invoke<FeVibeCheckConfig>(INVOKE.GET_CONFIG);
      // The page keeps its own copies of these, so bring them back in line
      setGeneral(pickGeneral(fresh));
      setIntensity(fresh.master_intensity);
      setDraft({
        bind: fresh.networking.bind,
        remote: fresh.networking.remote,
      });
      await refreshConfig();
      toast.info("Settings reset to defaults. Turn VibeCheck back on.");
    } catch (e) {
      toast.error(`Could not reset settings!\n${errorText(e)}`);
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
            <div className="mb-3 flex flex-wrap items-center gap-2">
              <TooltipLabel
                text="Connect to VRChat"
                tooltip={TOOLTIP.OscMode}
              />
              <div className="ml-auto flex gap-1 rounded-full border border-theme-border bg-theme-surface p-0.5">
                <ModeButton
                  active={mode === "OscQuery"}
                  onClick={() => switchMode("OscQuery")}
                >
                  Automatic (OSCQuery)
                </ModeButton>
                <ModeButton
                  active={mode === "Manual"}
                  onClick={() => switchMode("Manual")}
                >
                  Manual
                </ModeButton>
              </div>
            </div>
            {mode === "OscQuery" ? (
              <OscQueryPanel
                status={oscQueryStatus}
                port={config.networking.oscquery_port}
              />
            ) : (
              <form onSubmit={applyConnection}>
                <div className="grid grid-cols-[minmax(8rem,1fr)_minmax(8rem,2fr)] items-center gap-x-4 gap-y-2">
                  <TooltipLabel text="OSC Bind" tooltip={TOOLTIP.OSC_Bind} />
                  <AddressInput
                    name="bind"
                    value={draft.bind}
                    onChange={(bind) => setDraft((d) => ({ ...d, bind }))}
                  />
                  <TooltipLabel
                    text="OSC Remote"
                    tooltip={TOOLTIP.OSC_Remote}
                  />
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
            )}
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
            <div className="mt-3 grid grid-cols-[minmax(8rem,1fr)_minmax(8rem,2fr)] items-center gap-x-4">
              <TooltipLabel text="Scan length" tooltip={TOOLTIP.ScanLength} />
              <Select
                name="Scan length"
                value={scanLengthLabel(config.scan_length)}
                onValueChange={saveScanLength}
                options={scanLengthOptions}
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

          {(appExtensions.settingsSections ?? []).map(
            ({ id, title, hint, Body }) => (
              <Section key={id} title={title} hint={hint}>
                <Body />
              </Section>
            ),
          )}

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
            <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
              <div className="min-w-[12rem] flex-1">
                <div className="text-theme-text">Reset VibeCheck settings</div>
                <div className="text-xs text-theme-muted">
                  Puts every setting on this page back to its default and turns
                  VibeCheck off. Your toys&apos; settings are kept.
                </div>
              </div>
              <Button variant="danger" className="m-0" onClick={resetSettings}>
                {confirmSettingsReset ? "Click again to confirm" : "Reset"}
              </Button>
            </div>
            <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
              <div className="min-w-[12rem] flex-1">
                <div className="text-theme-text">Log files</div>
                <div className="text-xs text-theme-muted">
                  Attach VibeCheck.log to bug reports (VibeCheck.old.log is the
                  session before). Your home folder, computer name, IP and
                  Bluetooth addresses and VRChat IDs are left out.
                </div>
              </div>
              <Button
                variant="secondary"
                className="m-0"
                onClick={openLogFolder}
              >
                Open log folder
              </Button>
            </div>
          </Section>

          <Section title="About">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <div className="text-theme-text">
                  {appName} {version || "…"}
                </div>
                {updateStatus != "off" && (
                  <div
                    className={
                      canUpdate
                        ? "text-xs text-theme-success"
                        : "text-xs text-theme-muted"
                    }
                  >
                    {UPDATE_TEXT[updateStatus]}
                  </div>
                )}
              </div>
              {canUpdate && <UpdateButton enabled={canUpdate} />}
            </div>
          </Section>
        </div>
      </div>
    </div>
  );
}
