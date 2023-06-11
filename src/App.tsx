import { useToys } from "./hooks/useToys";
import VibecheckLogo from "./assets/VibeCheck_logo.png";
import cryingAnimeGirl from "./assets/menhera_chan.gif";
import Toy from "./features/Toy";
import { AnimatePresence } from "framer-motion";
import Button from "./layout/Button";
import { useCoreEvents } from "./hooks/useCoreEvents";
import Loading from "./components/Loading";
import { useState } from "react";
import { FeVCToy } from "../src-tauri/bindings/FeVCToy";
import Config from "./features/Config";
import { useUpdate } from "./hooks/useUpdate";
import { useVersion } from "./hooks/useVersion";
import UpdatePing from "./components/UpdatePing";
import { Cog6ToothIcon } from "@heroicons/react/24/solid";
import ExternalLogo from "./components/ExternalLogo";
import VrchatLogo from "./assets/vrchat-192x192.png";
import DiscordLogo from "./assets/discord-mark-white.svg";
import GithubLogo from "./assets/GitHub-Mark-Light-64px.png";
import "./App.css";
import Switch from "./layout/Switch";
import { cn } from "./utils";

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

  if (
    selection?.type == "Toy" &&
    !toysList.some((t) => `${t.toy_name} ${t.sub_id}` == selection.toyKey)
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
      />
    ) : !hasOnlineToys ? (
      <div className="flex-grow flex justify-center">
        <div className="flex flex-col justify-center items-center -mt-20">
          <img src={cryingAnimeGirl} />
          <div>No Online Toys</div>
        </div>
      </div>
    ) : null;

  function setToy(toy: FeVCToy) {
    const newKey = `${toy.toy_name} ${toy.sub_id}`;
    if (selection?.type == "Toy" && selection.toyKey == newKey) return;
    setSelection({
      type: "Toy",
      toyKey: `${toy.toy_name} ${toy.sub_id}`,
    });
  }

  function setConfig() {
    if (selection?.type == "Config") setSelection(null);
    else setSelection({ type: "Config" });
  }

  return (
    <div
      className="w-screen h-screen p-4"
      onContextMenu={(e) => e.preventDefault()}
    >
      <div className="grid grid-cols-[1fr,_4fr] h-[calc(100%-16px)] gap-3">
        <div className="flex flex-col gap-4">
          <img className="h-14 object-contain" src={VibecheckLogo} />
          <div>
            <div className="flex justify-around items-center">
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
          </div>
          <div className=" bg-gray-800 rounded-md justify-between flex flex-col max-h-fit flex-grow">
            <div className="flex flex-col overflow-y-scroll pl-2 scrollbar whitespace-nowrap">
              <AnimatePresence>
                {toysList.map((sidebarToy) => (
                  <button
                    key={`${sidebarToy.toy_name} ${sidebarToy.sub_id}`}
                    onClick={() => setToy(sidebarToy)}
                    className={cn(
                      selection?.type == "Toy" &&
                        `${sidebarToy.toy_name} ${sidebarToy.sub_id}` ==
                          selection.toyKey &&
                        "outline",
                      sidebarToy.toy_connected
                        ? "text-gray-200"
                        : "text-gray-500",
                      "bg-gray-700 rounded-md p-2 m-2 hover:bg-cyan-600 outline-2 outline-cyan-400"
                    )}
                  >
                    {sidebarToy.toy_name}
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
                  "h-10 cursor-pointer transform duration-300 ease-in-out"
                )}
                onClick={() => setConfig()}
              />
            </UpdatePing>
            <Switch
              theme="red-green"
              checked={isEnabled}
              onChange={toggleIsEnabled}
            />
          </div>
        </div>
        <div className="bg-gray-800 rounded-lg">
          <div className="flex justify-between p-4 h-full">{mainPanel}</div>
        </div>
      </div>
      <div className="m-1 text-center">{version}</div>
    </div>
  );
}
