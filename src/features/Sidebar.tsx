import ExternalLogo from "@/components/ExternalLogo";
import Loading from "@/components/Loading";
import ThemedLogo from "@/components/ThemedLogo";
import { SCAN_LENGTH, useCoreEventContext } from "@/context/CoreEvents";
import { TOOLTIP } from "@/data/constants";
import { toyKey } from "@/hooks/useToys";
import Button from "@/layout/Button";
import Switch from "@/layout/Switch";
import { cn } from "@/lib/utils";
import { mix } from "@/theme/color";
import ThemeIcon from "@/theme/ThemeIcon";
import { useTheme } from "@/theme/ThemeProvider";
import type { FeVCToy } from "@bindings/FeVCToy";
import type { ToyPower } from "@bindings/ToyPower";
import { Moon, Plug, Search, Settings } from "lucide-react";
import DiscordLogo from "../assets/discord-mark-white.svg";
import GithubLogo from "../assets/GitHub-Mark-Light-64px.png";
import VibecheckLogo from "../assets/VibeCheck_logo.png";
import VrchatLogo from "../assets/vrchat-192x192.png";

type SidebarProps = {
  toys: FeVCToy[];
  selectedToyKey: string | null;
  configSelected: boolean;
  onSelectToy: (toy: FeVCToy) => void;
  onToggleConfig: () => void;
  canUpdate: boolean;
  version: string;
};

/*
 * Left column: logo, app on/off status, toy list with scanning,
 * Settings, and a footer with community links and the version.
 */
export default function Sidebar({
  toys,
  selectedToyKey,
  configSelected,
  onSelectToy,
  onToggleConfig,
  canUpdate,
  version,
}: SidebarProps) {
  const connected = toys.filter((t) => t.toy_connected);
  const saved = toys.filter((t) => !t.toy_connected);

  return (
    <div className="flex min-h-0 flex-col gap-3">
      <ThemedLogo src={VibecheckLogo} className="h-14 object-contain" />

      <StatusCard />

      {/* Toy list + scan */}
      <div className="theme-panel theme-ears mt-2 flex min-h-0 flex-1 flex-col">
        <div className="scrollbar flex min-h-0 flex-1 select-none flex-col gap-3 overflow-y-auto px-2 pt-3">
          {toys.length === 0 ? (
            <EmptyToyList />
          ) : (
            <>
              {connected.length > 0 && (
                <ToyGroup
                  title="Connected"
                  toys={connected}
                  selectedToyKey={selectedToyKey}
                  onSelectToy={onSelectToy}
                />
              )}
              {saved.length > 0 && (
                <ToyGroup
                  title="Saved"
                  toys={saved}
                  selectedToyKey={selectedToyKey}
                  onSelectToy={onSelectToy}
                />
              )}
            </>
          )}
        </div>
        <ScanButton />
      </div>

      <SettingsRow
        selected={configSelected}
        canUpdate={canUpdate}
        onClick={onToggleConfig}
      />

      <div className="flex shrink-0 items-center justify-between px-1">
        <div className="flex items-center gap-3">
          <ExternalLogo
            src={VrchatLogo}
            tooltip={TOOLTIP.VrChatGroup}
            className="h-4"
          />
          <ExternalLogo
            src={DiscordLogo}
            tooltip={TOOLTIP.Discord}
            className="h-4"
          />
          <ExternalLogo
            src={GithubLogo}
            tooltip={TOOLTIP.Github}
            className="h-4"
          />
        </div>
        <span className="truncate pl-2 text-xs text-theme-muted">
          {version}
        </span>
      </div>
    </div>
  );
}

/** A: the app's on/off switch, labelled, with what VibeCheck is doing */
function StatusCard() {
  const { isEnabled, isScanning, toggleIsEnabled } = useCoreEventContext();
  const themeText = useTheme().theme.text;

  const status = !isEnabled
    ? "Turn on to connect to VRChat"
    : isScanning
      ? `${themeText.scanning} for toys…`
      : "Listening to VRChat";

  return (
    <label
      className={cn(
        "flex shrink-0 cursor-pointer items-center justify-between gap-2 rounded-2xl border px-3 py-2.5 text-left transition-colors duration-300",
        isEnabled
          ? "border-theme-success/50 bg-theme-success/10 shadow-theme-panel"
          : "border-theme-border bg-theme-surface",
      )}
    >
      <div className="min-w-0">
        <div className="flex items-center gap-2 font-semibold">
          <span className="relative flex h-2.5 w-2.5 shrink-0">
            {isEnabled && (
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-theme-success opacity-60" />
            )}
            <span
              className={cn(
                "relative inline-flex h-2.5 w-2.5 rounded-full",
                isEnabled ? "bg-theme-success" : "bg-theme-danger",
              )}
            />
          </span>
          <span
            className={isEnabled ? "text-theme-success" : "text-theme-text"}
          >
            {isEnabled ? "On" : "Off"}
          </span>
        </div>
        <div className="truncate text-xs text-theme-muted">{status}</div>
      </div>
      <Switch
        theme="red-green"
        checked={isEnabled}
        onCheckedChange={toggleIsEnabled}
        aria-label={isEnabled ? "Turn VibeCheck off" : "Turn VibeCheck on"}
      />
    </label>
  );
}

/** B: a titled group of toys with a count */
function ToyGroup({
  title,
  toys,
  selectedToyKey,
  onSelectToy,
}: {
  title: string;
  toys: FeVCToy[];
  selectedToyKey: string | null;
  onSelectToy: (toy: FeVCToy) => void;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center justify-between px-1 text-[0.7rem] font-semibold uppercase tracking-wider text-theme-muted">
        <span>{title}</span>
        <span>{toys.length}</span>
      </div>
      {toys.map((toy) => (
        <ToyEntry
          key={toyKey(toy)}
          toy={toy}
          selected={toyKey(toy) === selectedToyKey}
          onClick={() => onSelectToy(toy)}
        />
      ))}
    </div>
  );
}

function ToyEntry({
  toy,
  selected,
  onClick,
}: {
  toy: FeVCToy;
  selected: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      aria-pressed={selected}
      className={cn(
        "group flex items-center gap-2 rounded-2xl px-3 py-2 text-left text-sm transition-all duration-200 hover:-translate-y-0.5",
        selected
          ? "bg-gradient-to-r from-theme-primary/30 to-theme-secondary/30 ring-2 ring-theme-primary"
          : "bg-theme-raised hover:bg-theme-border",
        toy.toy_connected ? "text-theme-text" : "text-theme-muted",
      )}
    >
      {toy.toy_connected ? (
        <ThemeIcon className="h-4 w-4 shrink-0 text-theme-primary group-hover:animate-wiggle" />
      ) : (
        <Moon className="h-4 w-4 shrink-0 text-theme-muted" />
      )}
      <span className="min-w-0 flex-1 truncate">{toy.toy_name}</span>
      <PowerLabel power={toy.toy_power} />
    </button>
  );
}

/** Battery percentage (coloured like the battery icon) or a short state */
function PowerLabel({ power }: { power: ToyPower }) {
  const { colors } = useTheme().theme;
  if (power === "Offline") {
    return <span className="shrink-0 text-xs text-theme-muted">offline</span>;
  }
  if (power === "Pending") {
    return <span className="shrink-0 text-xs text-theme-muted">…</span>;
  }
  if (power === "NoBattery") {
    return <Plug className="h-3.5 w-3.5 shrink-0 text-theme-muted" />;
  }
  return (
    <span
      className="shrink-0 text-xs tabular-nums"
      style={{ color: mix(colors.danger, colors.success, power.Battery) }}
    >
      {Math.round(power.Battery * 100)}%
    </span>
  );
}

function EmptyToyList() {
  const themeText = useTheme().theme.text;
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-2 px-2 py-6 text-center">
      <ThemeIcon className="h-6 w-6 text-theme-muted" />
      <div className="text-sm text-theme-text">No toys yet</div>
      <div className="text-xs text-theme-muted">
        Use &ldquo;{themeText.scanButton}&rdquo; below to find some
      </div>
    </div>
  );
}

/** C: scan button with a bar that empties over the scan's duration */
function ScanButton() {
  const { isScanning, toggleScan } = useCoreEventContext();
  const themeText = useTheme().theme.text;

  return (
    <div className="shrink-0 px-2 pb-2 pt-2">
      <Button
        onClick={toggleScan}
        className="m-0 flex w-full items-center justify-center gap-2"
      >
        {isScanning ? (
          <>
            <span>{themeText.scanning}</span>
            <Loading />
          </>
        ) : (
          <>
            <Search className="h-4 w-4" />
            <span>{themeText.scanButton}</span>
          </>
        )}
      </Button>
      <div className="mt-1.5 h-1 overflow-hidden rounded-full bg-theme-border/50">
        {/* Mounted only while scanning, so the animation restarts each scan */}
        {isScanning && (
          <div
            className="h-full animate-scan-progress rounded-full bg-theme-primary"
            style={{ animationDuration: `${SCAN_LENGTH}ms` }}
          />
        )}
      </div>
    </div>
  );
}

/** D: Settings as a labelled navigation row */
function SettingsRow({
  selected,
  canUpdate,
  onClick,
}: {
  selected: boolean;
  canUpdate: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      aria-pressed={selected}
      className={cn(
        "group flex shrink-0 items-center gap-2 rounded-2xl border px-3 py-2 text-left text-sm transition-all duration-200",
        selected
          ? "border-theme-primary bg-gradient-to-r from-theme-primary/25 to-theme-secondary/25 text-theme-text"
          : "border-theme-border/60 bg-theme-surface text-theme-text hover:border-theme-secondary",
      )}
    >
      <Settings
        className={cn(
          "h-5 w-5 shrink-0 transition-transform duration-300",
          selected
            ? "rotate-45 text-theme-primary"
            : "text-theme-secondary group-hover:rotate-45",
        )}
      />
      <span className="flex-1 font-semibold">Settings</span>
      {canUpdate && (
        <span className="flex items-center gap-1 rounded-full bg-theme-success/15 px-2 py-0.5 text-[0.7rem] font-semibold text-theme-success">
          <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-theme-success" />
          Update
        </span>
      )}
    </button>
  );
}
