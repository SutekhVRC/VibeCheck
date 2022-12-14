import { useState } from "react";
import { useCoreEventContext } from "../../context/CoreEventContext";
import { useToys } from "../../context/ToysContext";
import NameInfo from "./NameInfo";
import ToySettingsModal from "./ToySettingsModal";
import { Feature } from "../Feature";
import Loading from "./Loading";
import { WrenchScrewdriverIcon } from "@heroicons/react/20/solid";
import cryingAnimeGirl from "../../assets/menhera_chan.gif";

export default function () {
  const [toySettingsModalIsOpen, setToySettingsModalIsOpen] = useState(false);
  const { toys } = useToys();
  const { isScanning, toggleScan } = useCoreEventContext();

  return (
    <div className="flex-col justify-between items-stretch text-lg rounded-lg p-4 bg-zinc-800 h-[600px]">
      <h1>
        <div className="grad-backwards text-clip text-4xl">Connected toys</div>
      </h1>
      {Object.keys(toys).length == 0 ? (
        <div className="flex justify-center items-center">
          No Toys
          <img src={cryingAnimeGirl} />
        </div>
      ) : (
        <div className="overflow-y-scroll pl-2 scrollbar pt-2 pb-2 max-h-[480px]">
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
              <ToySettingsModal
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
      <div>
        <button
          type="button"
          onClick={() => toggleScan()}
          className={
            "text-lg font-bold pl-4 pr-4 m-4 border-gray-500 border-solid border-2 rounded-sm shadow-zinc-900 shadow-md hover:border-gray-300"
          }
        >
          {isScanning ? (
            <div className="flex">
              <div>Scanning</div>
              <Loading />
            </div>
          ) : (
            <div>Search for toys</div>
          )}
        </button>
      </div>
    </div>
  );
}
