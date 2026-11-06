import ExternalLogo from "@/components/ExternalLogo";
import Loading from "@/components/Loading";
import ThemedLogo from "@/components/ThemedLogo";
import { SCAN_LENGTH, useCoreEventContext } from "@/context/CoreEvents";
import { TOOLTIP } from "@/data/constants";
import { toyKey } from "@/hooks/useToys";
import Button from "@/layout/Button";
import Switch from "@/layout/Switch";
import Tooltip from "@/layout/Tooltip";
import { cn } from "@/lib/utils";
import { mix } from "@/theme/color";
import PanelEars from "@/theme/PanelEars";
import ThemeIcon from "@/theme/ThemeIcon";
import { useTheme } from "@/theme/ThemeProvider";
import type { FeVCToy } from "@bindings/FeVCToy";
import type { ToyPower } from "@bindings/ToyPower";
import {
  Moon,
  PanelLeftClose,
  PanelLeftOpen,
  Plug,
  Power,
  Search,
  Settings,
} from "lucide-react";
import DiscordLogo from "../assets/discord-mark-white.svg";
import GithubLogo from "../assets/GitHub-Mark-Light-64px.png";
import VibecheckLogo from "../assets/VibeCheck_logo.png";
import VrchatLogo from "../assets/vrchat-192x192.png";

/**
 * Collapsed bar: every control is the same 44px rounded square on one centre
 * line, so the narrow bar reads as a single uniform column.
 */
const ICON_SQUARE =
  "flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl transition-all duration-200";

type SidebarProps = {
  toys: FeVCToy[];
  selectedToyKey: string | null;
  configSelected: boolean;
  onSelectToy: (toy: FeVCToy) => void;
  onToggleConfig: () => void;
  canUpdate: boolean;
  version: string;
  /** Icon-only narrow bar instead of the full sidebar */
  collapsed: boolean;
  onToggleCollapsed: () => void;
};

/*
 * Left column: logo, app on/off status, toy list with scanning,
 * Settings, and a footer with community links and the version.
 * Collapses into a narrow icon-only bar; tooltips then carry the labels.
 */
export default function Sidebar({
  toys,
  selectedToyKey,
  configSelected,
  onSelectToy,
  onToggleConfig,
  canUpdate,
  version,
  collapsed,
  onToggleCollapsed,
}: SidebarProps) {
  const connected = toys.filter((t) => t.toy_connected);
  const saved = toys.filter((t) => !t.toy_connected);

  return (
    <div
      className={cn(
        "flex min-h-0 min-w-0 flex-col gap-3",
        collapsed && "items-center",
      )}
    >
      {collapsed ? (
        // The logo tile doubles as the expand button: it shows the theme
        // icon, and swaps to the expand icon on hover
        <Tooltip text="Expand sidebar" side="right">
          <button
            type="button"
            onClick={onToggleCollapsed}
            aria-label="Expand sidebar"
            aria-expanded={false}
            className={cn(
              ICON_SQUARE,
              "group bg-theme-primary/15 text-theme-primary hover:bg-theme-primary/25",
            )}
          >
            <ThemeIcon className="h-6 w-6 group-hover:hidden" />
            <PanelLeftOpen className="hidden h-6 w-6 group-hover:block" />
          </button>
        </Tooltip>
      ) : (
        <div className="flex shrink-0 items-center justify-between gap-1">
          <ThemedLogo
            src={VibecheckLogo}
            className="h-14 min-w-0 flex-1 object-contain"
          />
          <Tooltip text="Collapse sidebar" side="bottom">
            <button
              type="button"
              onClick={onToggleCollapsed}
              aria-label="Collapse sidebar"
              aria-expanded={true}
              className="rounded-lg p-1.5 text-theme-muted transition-colors hover:bg-theme-raised hover:text-theme-primary"
            >
              <PanelLeftClose className="h-4 w-4" />
            </button>
          </Tooltip>
        </div>
      )}

      <StatusCard collapsed={collapsed} />

      {/* Toy list + scan. Ears only fit on the full-width panel. */}
      <div
        className={cn(
          "theme-panel flex min-h-0 flex-1 flex-col",
          // Collapsed: 44px squares + 8px padding + 1px border on each side
          collapsed ? "w-[3.875rem]" : "theme-ears mt-2",
        )}
      >
        {!collapsed && <PanelEars />}
        <ToyListHeader collapsed={collapsed} />
        <div
          className={cn(
            // overflow-x-hidden: the list only ever scrolls vertically, so
            // never show a horizontal scrollbar
            "scrollbar flex min-h-0 flex-1 select-none flex-col overflow-y-auto overflow-x-hidden pb-2 pt-3",
            collapsed ? "items-center gap-2 px-2" : "gap-3 px-2",
          )}
        >
          {toys.length === 0 ? (
            !collapsed && <EmptyToyList />
          ) : collapsed ? (
            <>
              {connected.map((toy) => (
                <ToyIconButton
                  key={toyKey(toy)}
                  toy={toy}
                  selected={toyKey(toy) === selectedToyKey}
                  onClick={() => onSelectToy(toy)}
                />
              ))}
              {connected.length > 0 && saved.length > 0 && (
                <div className="my-0.5 h-px w-8 shrink-0 bg-theme-border" />
              )}
              {saved.map((toy) => (
                <ToyIconButton
                  key={toyKey(toy)}
                  toy={toy}
                  selected={toyKey(toy) === selectedToyKey}
                  onClick={() => onSelectToy(toy)}
                />
              ))}
            </>
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
      </div>

      <SettingsRow
        selected={configSelected}
        canUpdate={canUpdate}
        onClick={onToggleConfig}
        collapsed={collapsed}
      />

      {/* Community links and version: only in the full sidebar. The
          collapsed bar keeps to the core controls; the links are one click
          away by expanding, and the version is also in Settings > About. */}
      {!collapsed && (
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
      )}
    </div>
  );
}

/** A: the app's on/off switch, labelled, with what VibeCheck is doing */
function StatusCard({ collapsed }: { collapsed: boolean }) {
  const { isEnabled, isScanning, toggleIsEnabled } = useCoreEventContext();
  const themeText = useTheme().theme.text;

  const status = !isEnabled
    ? "Turn on to connect to VRChat"
    : isScanning
      ? `${themeText.scanning} for toys…`
      : "Listening to VRChat";

  const dot = (
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
  );

  const toggle = (
    <Switch
      theme="red-green"
      checked={isEnabled}
      onCheckedChange={toggleIsEnabled}
      aria-label={isEnabled ? "Turn VibeCheck off" : "Turn VibeCheck on"}
    />
  );

  const cardClass = cn(
    "flex shrink-0 cursor-pointer rounded-2xl border transition-colors duration-300",
    isEnabled
      ? "border-theme-success/50 bg-theme-success/10 shadow-theme-panel"
      : "border-theme-border bg-theme-surface",
  );

  if (collapsed) {
    // A square power button instead of a switch, so it matches the column
    return (
      <Tooltip
        text={`${isEnabled ? "On" : "Off"}: ${status}. Click to turn ${
          isEnabled ? "off" : "on"
        }.`}
        side="right"
      >
        <button
          type="button"
          onClick={toggleIsEnabled}
          aria-pressed={isEnabled}
          aria-label={isEnabled ? "Turn VibeCheck off" : "Turn VibeCheck on"}
          className={cn(
            ICON_SQUARE,
            "relative ring-1",
            isEnabled
              ? "bg-theme-success/15 text-theme-success shadow-theme-panel ring-theme-success/50"
              : "bg-theme-danger/10 text-theme-danger ring-theme-danger/40 hover:bg-theme-danger/15",
          )}
        >
          <Power className="h-5 w-5" strokeWidth={2.5} />
          {isEnabled && (
            <span className="absolute right-1.5 top-1.5 flex h-2 w-2">
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-theme-success opacity-60" />
              <span className="relative inline-flex h-2 w-2 rounded-full bg-theme-success" />
            </span>
          )}
        </button>
      </Tooltip>
    );
  }

  return (
    <label
      className={cn(
        cardClass,
        "items-center justify-between gap-2 px-3 py-2.5 text-left",
      )}
    >
      <div className="min-w-0">
        <div className="flex items-center gap-2 font-semibold">
          {dot}
          <span
            className={isEnabled ? "text-theme-success" : "text-theme-text"}
          >
            {isEnabled ? "On" : "Off"}
          </span>
        </div>
        <div className="truncate text-xs text-theme-muted">{status}</div>
      </div>
      {toggle}
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

/** Collapsed toy: icon button, battery as a coloured dot, details in tooltip */
function ToyIconButton({
  toy,
  selected,
  onClick,
}: {
  toy: FeVCToy;
  selected: boolean;
  onClick: () => void;
}) {
  const { colors } = useTheme().theme;
  const power = toy.toy_power;
  const battery = typeof power === "object" ? power.Battery : null;
  const powerText =
    battery !== null
      ? `${Math.round(battery * 100)}%`
      : power === "NoBattery"
        ? "no battery"
        : power === "Pending"
          ? "battery pending"
          : "offline";

  return (
    <Tooltip text={`${toy.toy_name} · ${powerText}`} side="right">
      <button
        onClick={onClick}
        aria-pressed={selected}
        aria-label={toy.toy_name}
        className={cn(
          ICON_SQUARE,
          "group relative",
          selected
            ? "bg-gradient-to-br from-theme-primary/30 to-theme-secondary/30 ring-2 ring-theme-primary"
            : "bg-theme-raised hover:bg-theme-border",
        )}
      >
        {toy.toy_connected ? (
          <ThemeIcon className="h-5 w-5 text-theme-primary group-hover:animate-wiggle" />
        ) : (
          <Moon className="h-5 w-5 text-theme-muted" />
        )}
        {battery !== null && (
          <span
            className="absolute bottom-1 right-1 h-2 w-2 rounded-full ring-2 ring-theme-raised"
            style={{
              background: mix(colors.danger, colors.success, battery),
            }}
          />
        )}
      </button>
    </Tooltip>
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
        Use &ldquo;{themeText.scanButton}&rdquo; above to find some
      </div>
    </div>
  );
}

/** C: scan button with a bar that empties over the scan's duration */
function ToyListHeader({ collapsed }: { collapsed: boolean }) {
  const { isScanning, toggleScan } = useCoreEventContext();
  const themeText = useTheme().theme.text;

  const progress = (
    <div
      className={cn(
        "h-1 overflow-hidden rounded-full bg-theme-border/50",
        collapsed ? "mt-1.5 w-11" : "mt-2 w-full",
      )}
    >
      {/* Mounted only while scanning, so the animation restarts each scan */}
      {isScanning && (
        <div
          className="h-full animate-scan-progress rounded-full bg-theme-primary"
          style={{ animationDuration: `${SCAN_LENGTH}ms` }}
        />
      )}
    </div>
  );

  if (collapsed) {
    return (
      <div className="flex shrink-0 flex-col items-center px-2 pt-2">
        <Tooltip
          text={isScanning ? `${themeText.scanning}…` : themeText.scanButton}
          side="right"
        >
          <Button
            onClick={toggleScan}
            aria-label={isScanning ? "Stop scanning" : themeText.scanButton}
            className={cn(ICON_SQUARE, "m-0 px-0 py-0")}
          >
            <Search className={cn("h-5 w-5", isScanning && "animate-pulse")} />
          </Button>
        </Tooltip>
        {progress}
      </div>
    );
  }

  return (
    <div className="shrink-0 px-3 pt-3">
      <div className="flex items-center justify-between gap-2">
        <span className="text-sm font-semibold text-theme-text">Toys</span>
        <Tooltip
          text={isScanning ? "Stop scanning" : "Search for Bluetooth toys"}
        >
          <Button
            onClick={toggleScan}
            className="m-0 flex min-w-0 items-center gap-1.5 px-3 py-0.5 text-sm"
          >
            {isScanning ? (
              <>
                <span className="truncate">{themeText.scanning}</span>
                <Loading />
              </>
            ) : (
              <>
                <Search className="h-3.5 w-3.5 shrink-0" />
                <span className="truncate">{themeText.scanButton}</span>
              </>
            )}
          </Button>
        </Tooltip>
      </div>
      {progress}
    </div>
  );
}

/** D: Settings as a labelled navigation row (an icon button when collapsed) */
function SettingsRow({
  selected,
  canUpdate,
  onClick,
  collapsed,
}: {
  selected: boolean;
  canUpdate: boolean;
  onClick: () => void;
  collapsed: boolean;
}) {
  const rowClass = cn(
    "group flex shrink-0 items-center rounded-2xl border text-sm transition-all duration-200",
    selected
      ? "border-theme-primary bg-gradient-to-r from-theme-primary/25 to-theme-secondary/25 text-theme-text"
      : "border-theme-border/60 bg-theme-surface text-theme-text hover:border-theme-secondary",
  );
  const gear = (
    <Settings
      className={cn(
        "h-5 w-5 shrink-0 transition-transform duration-300",
        selected
          ? "rotate-45 text-theme-primary"
          : "text-theme-secondary group-hover:rotate-45",
      )}
    />
  );

  if (collapsed) {
    return (
      <Tooltip
        text={canUpdate ? "Settings · update available" : "Settings"}
        side="right"
      >
        <button
          onClick={onClick}
          aria-pressed={selected}
          aria-label="Settings"
          // Same square and states as a collapsed toy button
          className={cn(
            ICON_SQUARE,
            "group relative",
            selected
              ? "bg-gradient-to-br from-theme-primary/30 to-theme-secondary/30 ring-2 ring-theme-primary"
              : "bg-theme-raised hover:bg-theme-border",
          )}
        >
          {gear}
          {canUpdate && (
            <span className="absolute right-1.5 top-1.5 h-2 w-2 animate-pulse rounded-full bg-theme-success ring-2 ring-theme-raised" />
          )}
        </button>
      </Tooltip>
    );
  }

  return (
    <button
      onClick={onClick}
      aria-pressed={selected}
      className={cn(rowClass, "gap-2 px-3 py-2 text-left")}
    >
      {gear}
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
