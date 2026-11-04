import { FeVCToy } from "@bindings/FeVCToy";
import { AnimatePresence } from "framer-motion";
import { Moon, PawPrint, Settings } from "lucide-react";
import { useState } from "react";
import "./App.css";
import GithubLogo from "./assets/GitHub-Mark-Light-64px.png";
import VibecheckLogo from "./assets/VibeCheck_logo.png";
import DiscordLogo from "./assets/discord-mark-white.svg";
import cryingAnimeGirl from "./assets/menhera_chan.gif";
import VrchatLogo from "./assets/vrchat-192x192.png";
import ExternalLogo from "./components/ExternalLogo";
import Loading from "./components/Loading";
import UpdatePing from "./components/UpdatePing";
import { useCoreEventContext } from "./context/CoreEvents";
import { TOOLTIP } from "./data/constants";
import Config from "./features/Config";
import Toy from "./features/Toy";
import { toyKey, useToys } from "./hooks/useToys";
import { useUpdate } from "./hooks/useUpdate";
import { useVersion } from "./hooks/useVersion";
import Button from "./layout/Button";
import Switch from "./layout/Switch";
import Tooltip from "./layout/Tooltip";
import { cn } from "./lib/utils";

type Selection = ToySelection | ConfigSelection | null;

type ToySelection = {
  type: "Toy";
  toyKey: string;
};

type ConfigSelection = {
  type: "Config";
};

export default function App() {
  const [selection, setSelection] = useState<Selection>(null);
  const { toys, hasOnlineToys } = useToys();
  const toy =
    selection?.type == "Toy" && selection.toyKey in toys
      ? toys[selection.toyKey]
      : null;
  const toysList = Object.values(toys);
  const {
    isScanning,
    toggleScan,
    isEnabled,
    toggleIsEnabled,
    config,
    refreshConfig,
  } = useCoreEventContext();
  const { canUpdate } = useUpdate();
  const { version } = useVersion();

  async function disableOnPortChange() {
    if (isEnabled) {
      await toggleIsEnabled();
    }
  }

  if (
    selection?.type == "Toy" &&
    !toysList.some((t) => toyKey(t) == selection.toyKey)
  )
    setSelection(null); // selection is no longer valid

  const mainPanel =
    selection?.type == "Toy" && toy != null ? (
      <Toy toy={toy} />
    ) : selection?.type == "Config" && config != null ? (
      <Config
        config={config}
        refreshConfig={refreshConfig}
        canUpdate={canUpdate}
        disableOnPortChange={disableOnPortChange}
      />
    ) : !hasOnlineToys ? (
      <div className="flex w-full flex-col items-center justify-center gap-3">
        <img
          src={cryingAnimeGirl}
          className="animate-float rounded-3xl drop-shadow-[0_0_25px_rgba(247,178,217,0.35)]"
        />
        <div className="flex items-center gap-2 text-lg text-cute-pink">
          <PawPrint className="h-5 w-5" />
          No online toys... yet!
          <PawPrint className="h-5 w-5 -scale-x-100" />
        </div>
        <div className="text-sm text-cute-muted">
          Press &ldquo;Look for toys&rdquo; to sniff some out
        </div>
      </div>
    ) : null;

  function setToy(toy: FeVCToy) {
    const newKey = toyKey(toy);
    if (selection?.type == "Toy" && selection.toyKey == newKey) return;
    setSelection({
      type: "Toy",
      toyKey: newKey,
    });
  }

  function setConfig() {
    if (selection?.type == "Config") setSelection(null);
    else setSelection({ type: "Config" });
  }

  return (
    <div
      className="h-screen w-full p-4"
      onContextMenu={(e) => e.preventDefault()}
    >
      <div className="grid grid-cols-[12rem,_1fr] gap-3">
        <div className="flex flex-col gap-1">
          <img
            className="h-14 object-contain drop-shadow-[0_0_12px_rgba(247,178,217,0.45)]"
            src={VibecheckLogo}
          />
          <div className="mb-1 mt-2 flex items-center justify-around">
            <ExternalLogo src={VrchatLogo} tooltip={TOOLTIP.VrChatGroup} />
            <ExternalLogo src={DiscordLogo} tooltip={TOOLTIP.Discord} />
            <ExternalLogo src={GithubLogo} tooltip={TOOLTIP.Github} />
          </div>
          <div className="cute-panel cute-ears mt-3 flex h-[calc(100vh-197px)] flex-col justify-between">
            <div className="scrollbar flex select-none flex-col overflow-y-auto px-1 pt-2">
              <AnimatePresence>
                {toysList.map((sidebarToy) => (
                  <button
                    key={toyKey(sidebarToy)}
                    onClick={() => setToy(sidebarToy)}
                    className={cn(
                      selection?.type == "Toy" &&
                        toyKey(sidebarToy) == selection.toyKey
                        ? "bg-gradient-to-r from-cute-pink/30 to-cute-lilac/30 ring-2 ring-cute-pink"
                        : "bg-cute-raised hover:bg-cute-line",
                      sidebarToy.toy_connected
                        ? "text-cute-cream"
                        : "text-cute-muted",
                      "group m-1.5 flex items-center gap-2 rounded-2xl px-3 py-2 text-left transition-all duration-200 hover:-translate-y-0.5",
                    )}
                  >
                    {sidebarToy.toy_connected ? (
                      <PawPrint className="h-4 w-4 shrink-0 text-cute-pink group-hover:animate-wiggle" />
                    ) : (
                      <Moon className="h-4 w-4 shrink-0 text-cute-muted" />
                    )}
                    <span className="truncate">{sidebarToy.toy_name}</span>
                  </button>
                ))}
              </AnimatePresence>
            </div>
            <Button onClick={toggleScan}>
              {isScanning ? (
                <div className="flex items-center justify-center">
                  <div>Sniffing</div>
                  <Loading />
                </div>
              ) : (
                <div>Look for toys</div>
              )}
            </Button>
          </div>
          <div className="flex items-center justify-around">
            <UpdatePing canUpdate={canUpdate}>
              <Settings
                className={cn(
                  selection?.type == "Config"
                    ? "rotate-45 text-cute-pink"
                    : "text-cute-lilac hover:text-cute-pink",
                  "h-10 transform cursor-pointer duration-300 ease-in-out",
                )}
                onClick={() => setConfig()}
              />
            </UpdatePing>
            <Tooltip
              text={isEnabled ? "Disable App" : "Enable App"}
              asChild={false}
            >
              <Switch
                theme="red-green"
                checked={isEnabled}
                onCheckedChange={toggleIsEnabled}
              />
            </Tooltip>
          </div>
        </div>
        <div className="cute-panel cute-ears mt-3">
          <div className="flex h-full p-4">{mainPanel}</div>
        </div>
      </div>
      <div className="mt-1 text-xs text-cute-muted">{version}</div>
    </div>
  );
}
