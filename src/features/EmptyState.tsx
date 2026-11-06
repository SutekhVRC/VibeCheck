import { useCoreEventContext } from "@/context/CoreEvents";
import { INVOKE } from "@/data/constants";
import Button from "@/layout/Button";
import Switch from "@/layout/Switch";
import { cn } from "@/lib/utils";
import ThemeIcon from "@/theme/ThemeIcon";
import { useTheme } from "@/theme/ThemeProvider";
import type { FeBrowserLink } from "@bindings/FeBrowserLink";
import { invoke } from "@tauri-apps/api/core";
import { BookOpen, Check, Search } from "lucide-react";
import { ReactNode } from "react";
import { toast } from "sonner";

/*
 * Main panel when no toys are online: the theme's icon (breathing when idle,
 * rippling like radar while scanning) above a getting-started checklist that
 * ticks itself off as the app is turned on and toys are found.
 */
export default function EmptyState() {
  const {
    isEnabled,
    isScanning,
    toggleIsEnabled,
    toggleScan,
    autoEnableOnScan,
  } = useCoreEventContext();
  const themeText = useTheme().theme.text;

  // This screen only shows while no toys are online, so the first unfinished
  // step is turning the app on, then finding a toy.
  const current = !isEnabled ? 1 : 2;

  return (
    <div className="flex w-full flex-col items-center justify-center gap-6 overflow-y-auto px-4 py-6">
      <SearchIcon state={!isEnabled ? "off" : isScanning ? "scanning" : "on"} />

      <div className="text-center">
        <div className="text-lg font-semibold text-theme-text">
          {themeText.noToysTitle}
        </div>
        <div className="text-sm text-theme-muted">
          {isScanning ? `${themeText.scanning}…` : themeText.noToysHint}
        </div>
      </div>

      <ol className="flex w-full max-w-sm flex-col gap-1 rounded-2xl border border-theme-border/60 bg-theme-raised p-2 text-left text-sm">
        <Step
          number={1}
          state={isEnabled ? "done" : "current"}
          title="Turn on VibeCheck"
          detail="Connects to VRChat over OSC"
          action={
            <Switch
              theme="red-green"
              checked={isEnabled}
              onCheckedChange={toggleIsEnabled}
              aria-label={
                isEnabled ? "Turn VibeCheck off" : "Turn VibeCheck on"
              }
            />
          }
        />
        <Step
          number={2}
          state={current === 2 ? "current" : "upcoming"}
          title="Connect a toy"
          detail={
            isScanning
              ? "Searching for Bluetooth toys"
              : !isEnabled && !autoEnableOnScan
                ? "Turn VibeCheck on first"
                : "Switch your toy on, then look for it"
          }
          action={
            <Button
              onClick={toggleScan}
              variant={current === 2 ? "primary" : "secondary"}
              className="m-0 flex items-center gap-1.5 whitespace-nowrap px-3 py-0.5 text-sm"
            >
              <Search className="h-3.5 w-3.5" />
              {isScanning ? "Stop" : themeText.scanButton}
            </Button>
          }
        />
        <Step
          number={3}
          state="upcoming"
          title="Set up your avatar"
          detail="Add contacts or SPS/TPS so VRChat can drive your toys"
          action={<GuidesButton />}
        />
      </ol>
    </div>
  );
}

/** The theme's icon in a glowing circle */
function SearchIcon({ state }: { state: "off" | "on" | "scanning" }) {
  return (
    <div className="relative flex h-28 w-28 items-center justify-center">
      {state === "scanning" &&
        [0, 0.8, 1.6].map((delay) => (
          <span
            key={delay}
            className="absolute inset-0 animate-radar rounded-full border-2 border-theme-primary"
            style={{ animationDelay: `${delay}s` }}
          />
        ))}
      <div
        className={cn(
          "relative flex h-24 w-24 items-center justify-center rounded-full border-2 transition-colors duration-500",
          state === "off"
            ? "border-theme-border bg-theme-raised text-theme-muted"
            : "border-theme-primary/40 bg-theme-primary/10 text-theme-primary shadow-theme-panel",
          state === "on" && "animate-float",
        )}
      >
        <ThemeIcon className="h-10 w-10" />
      </div>
    </div>
  );
}

function Step({
  number,
  state,
  title,
  detail,
  action,
}: {
  number: number;
  state: "done" | "current" | "upcoming";
  title: string;
  detail: string;
  action?: ReactNode;
}) {
  return (
    <li
      className={cn(
        "flex items-center gap-3 rounded-xl px-2 py-2",
        state === "current" && "bg-theme-primary/10",
      )}
    >
      <span
        className={cn(
          "flex h-6 w-6 shrink-0 items-center justify-center rounded-full border-2 text-xs font-bold",
          state === "done" &&
            "border-theme-success bg-theme-success text-theme-on-accent",
          state === "current" && "border-theme-primary text-theme-primary",
          state === "upcoming" && "border-theme-border text-theme-muted",
        )}
      >
        {state === "done" ? (
          <Check className="h-3.5 w-3.5" strokeWidth={3} />
        ) : (
          number
        )}
      </span>
      <div className="min-w-0 flex-1">
        <div
          className={cn(
            "font-semibold",
            state === "done"
              ? "text-theme-muted line-through"
              : "text-theme-text",
          )}
        >
          {title}
        </div>
        <div className="text-xs text-theme-muted">{detail}</div>
      </div>
      {action && <div className="shrink-0">{action}</div>}
    </li>
  );
}

/*
 * Opens the VibeCheck wiki, which has every setup guide (avatar setup,
 * connecting toys, app setup).
 */
function GuidesButton() {
  async function openGuides() {
    try {
      const link: FeBrowserLink = "GithubWiki";
      await invoke(INVOKE.OPEN_BROWSER, { link });
    } catch (e) {
      toast.error(`Could not open browser\n${JSON.stringify(e)}`);
    }
  }
  return (
    <Button
      variant="secondary"
      onClick={openGuides}
      className="m-0 flex items-center gap-1.5 whitespace-nowrap px-3 py-0.5 text-sm"
    >
      <BookOpen className="h-3.5 w-3.5" />
      Guides
    </Button>
  );
}
