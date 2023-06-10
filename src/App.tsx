import { useToys } from "./hooks/useToys";
import VibecheckLogo from "./assets/VibeCheck_logo.png";
import cryingAnimeGirl from "./assets/menhera_chan.gif";
import Toy from "./features/Toy";
import { AnimatePresence } from "framer-motion";
import Button from "./layout/Button";
import { useCoreEventContext } from "./context/CoreEventContext";
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
  const { toys } = useToys();
  const toy =
    selection?.type == "Toy" && selection.toyKey in toys
      ? toys[selection.toyKey]
      : null;
  const toysList = Object.values(toys);
  if (
    selection?.type == "Toy" &&
    !toysList.some((t) => `${t.toy_name} ${t.sub_id}` == selection.toyKey)
  )
    setSelection(null); // selection is no longer valid
  const { isScanning, toggleScan, isEnabled, toggleIsEnabled } =
    useCoreEventContext();
  const { canUpdate } = useUpdate();
  const { version } = useVersion();

  function setToy(toy: FeVCToy) {
    setSelection({
      type: "Toy",
      toyKey: `${toy.toy_name} ${toy.sub_id}`,
    });
  }

  function setConfig() {
    setSelection({ type: "Config" });
  }

  return (
    <div className="w-screen h-screen p-4">
      <div className="grid grid-cols-[1fr,_4fr] h-[calc(100%-40px)] gap-3">
        <div className="flex flex-col gap-2">
          <img className="h-16 object-contain" src={VibecheckLogo} />
          <div className=" bg-gray-800 rounded-md justify-between flex flex-col max-h-fit flex-grow">
            {toysList.length === 0 ? (
              <div className="flex-grow flex justify-center">
                <div className="flex flex-col justify-center items-center -mt-20">
                  <img src={cryingAnimeGirl} />
                  <div>No Toys</div>
                </div>
              </div>
            ) : (
              <div className="flex flex-col overflow-y-scroll pl-2 scrollbar whitespace-nowrap">
                <AnimatePresence>
                  {toysList.map((toy) => (
                    <button
                      key={`${toy.toy_name} ${toy.sub_id}`}
                      onClick={() => setToy(toy)}
                      className={cn(
                        selection?.type == "Toy" &&
                          `${toy.toy_name} ${toy.sub_id}` == selection.toyKey &&
                          "outline",
                        toy.toy_connected ? "text-gray-200" : "text-gray-500",
                        "bg-gray-700 rounded-md p-2 m-2 hover:bg-cyan-600 outline-2 outline-cyan-400"
                      )}
                    >
                      {toy.toy_name}
                    </button>
                  ))}
                </AnimatePresence>
              </div>
            )}
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
        </div>
        <div className="bg-gray-800 rounded-lg">
          <div className="flex justify-between p-4 h-full">
            {toy != null ? (
              <Toy toy={toy} />
            ) : selection?.type == "Config" ? (
              <Config />
            ) : null}
          </div>
        </div>
      </div>
      <div className="m-2">
        <div className="grid grid-cols-3 items-center">
          <div className="flex justify-around items-center">
            <UpdatePing canUpdate={canUpdate}>
              <Cog6ToothIcon
                className={cn(
                  { "rotate-45": selection?.type == "Config" },
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
          {version}
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
      </div>
    </div>
  );
}
