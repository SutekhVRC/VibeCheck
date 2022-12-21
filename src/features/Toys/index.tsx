import { useState } from "react";
import { useCoreEventContext } from "../../context/CoreEventContext";
import { useToys } from "../../context/ToysContext";
import NameInfo from "./NameInfo";
import Settings from "./Settings";
import { Feature } from "./Feature";
import { WrenchScrewdriverIcon } from "@heroicons/react/20/solid";
import cryingAnimeGirl from "../../assets/menhera_chan.gif";
import ScanButton from "../../components/ScanButton";

export default function () {
  const [toySettingsModalIsOpen, setToySettingsModalIsOpen] = useState(false);
  const { toys } = useToys();
  const { isScanning, toggleScan } = useCoreEventContext();

  return (
    <div className="flex-col justify-between items-stretch text-lg rounded-lg p-4 bg-zinc-800 h-[600px]">
      {Object.keys(toys).length == 0 ? (
        <div className="flex justify-center items-center">
          No Toys
          <img src={cryingAnimeGirl} />
        </div>
      ) : (
        <div className="overflow-y-scroll pl-2 scrollbar pt-2 pb-2 max-h-[520px]">
          {Object.values(toys).map((toy) => (
            <div
              className="rounded-md bg-zinc-700 p-4 m-2"
              key={`${toy.toy_name} ${toy.toy_id}`}
            >
              <div className="text-4xl flex justify-between items-center">
                <NameInfo name={toy.toy_name} battery={toy.battery_level} />
                <WrenchScrewdriverIcon
                  className="h-6 cursor-pointer"
                  onClick={() => setToySettingsModalIsOpen(true)}
                />
              </div>
              <Settings
                isOpen={toySettingsModalIsOpen}
                onClose={() => setToySettingsModalIsOpen(false)}
                toy={toy}
              />
              <div className="grid">
                {toy.features.map((feature) => (
                  <div
                    className="flex flex-col"
                    key={`${toy.toy_id} ${feature.feature_type} ${feature.feature_index}`}
                  >
                    <hr className="border-1 border-zinc-800 m-1 border-opacity-75" />
                    <Feature toyId={toy.toy_id} feature={feature} />
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
      <ScanButton isScanning={isScanning} toggleScan={toggleScan} />
    </div>
  );
}
