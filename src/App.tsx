import { Cog6ToothIcon } from "@heroicons/react/24/solid";
import { AnimatePresence } from "framer-motion";
import { useState } from "react";
import { FeVCToy } from "../src-tauri/bindings/FeVCToy";
import "./App.css";
import GithubLogo from "./assets/GitHub-Mark-Light-64px.png";
import VibecheckLogo from "./assets/VibeCheck_logo.png";
import DiscordLogo from "./assets/discord-mark-white.svg";
import cryingAnimeGirl from "./assets/menhera_chan.gif";
import VrchatLogo from "./assets/vrchat-192x192.png";
import ExternalLogo from "./components/ExternalLogo";
import Loading from "./components/Loading";
import UpdatePing from "./components/UpdatePing";
import Config from "./features/Config";
import Toy from "./features/Toy";
import { useCoreEvents } from "./hooks/useCoreEvents";
import { parseName, toyKey, useToys } from "./hooks/useToys";
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
  } = useCoreEvents();
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
      <div className="flex flex-col justify-center items-center w-full">
        <img src={cryingAnimeGirl} />
        <div>No Online Toys</div>
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
      className="w-full h-screen p-4"
      onContextMenu={(e) => e.preventDefault()}
    >
      <div className="grid grid-cols-[12rem,_1fr] gap-3">
        <div className="flex flex-col gap-1">
          <img className="h-14 object-contain" src={VibecheckLogo} />
          <div className="flex justify-around items-center mt-2 mb-1">
            <ExternalLogo
              src={VrchatLogo}
              link="VRChatGroup"
              tooltip="Vibecheck VRChat Group"
            />
            <ExternalLogo
              src={DiscordLogo}
              link="Discord"
              tooltip="Vibecheck Discord"
            />
            <ExternalLogo
              src={GithubLogo}
              link="Github"
              tooltip="Vibecheck Github"
            />
          </div>
          <div className="bg-zinc-800 rounded-md justify-between flex-col overflow-hidden h-[calc(100vh-185px)] flex">
            <div className="flex flex-col pl-2 select-none overflow-y-scroll scrollbar">
              <AnimatePresence>
                {toysList.map((sidebarToy) => (
                  <button
                    key={toyKey(sidebarToy)}
                    onClick={() => setToy(sidebarToy)}
                    className={cn(
                      selection?.type == "Toy" &&
                        toyKey(sidebarToy) == selection.toyKey &&
                        "outline",
                      sidebarToy.toy_connected
                        ? "text-zinc-200"
                        : "text-zinc-500",
                      "bg-zinc-700 rounded-md p-2 m-2 hover:bg-cyan-600 outline-2 outline-cyan-400",
                    )}
                  >
                    {parseName(sidebarToy.toy_name)}
                  </button>
                ))}
              </AnimatePresence>
            </div>
            <Button onClick={toggleScan}>
              {isScanning ? (
                <div className="flex justify-center">
                  <div>Scanning</div>
                  <Loading />
                </div>
              ) : (
                <div>Look for toys</div>
              )}
            </Button>
          </div>
          <div className="flex justify-around items-center">
            <UpdatePing canUpdate={canUpdate}>
              <Cog6ToothIcon
                className={cn(
                  selection?.type == "Config" && "rotate-45",
                  "h-10 cursor-pointer transform duration-300 ease-in-out",
                )}
                onClick={() => setConfig()}
              />
            </UpdatePing>
            <Tooltip
              text={isEnabled ? "Disable OSC" : "Enable OSC"}
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
        <div className="bg-zinc-800 rounded-lg">
          <div className="flex p-4 h-full">{mainPanel}</div>
        </div>
      </div>
      {version}
    </div>
  );
}
