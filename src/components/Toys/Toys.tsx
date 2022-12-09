import { useState } from "react";
import { useCoreEventContext } from "../../context/CoreEventContext";
import { useToys } from "../../context/ToysContext";
import { percent } from "../../utils";
import NameBadge from "./NameBadge";
import ToySettingsModal from "./ToySettingsModal";
import { Feature } from "../Feature";
import Loading from "./Loading";

export default function () {
  const [toySettingsModalIsOpen, setToySettingsModalIsOpen] = useState(false);
  const { toys } = useToys();
  const { isScanning, toggleScan } = useCoreEventContext();

  return (
    <div className="flex-col justify-between items-stretch text-lg rounded-lg p-8 m-1 bg-zinc-800">
      <h1>
        <div className="grad-backwards text-clip text-4xl">Connected toys</div>
      </h1>
      <div className="overflow-y-auto">
        {Object.values(toys).map((toy) => (
          <div
            className="rounded-md p-6 bg-zinc-700"
            key={`${toy.toy_name} ${toy.toy_id}`}
          >
            <div
              className="text-4xl flex justify-between items-center cursor-pointer"
              onClick={() => setToySettingsModalIsOpen(true)}
            >
              <NameBadge name={toy.toy_name} />
              {toy.battery_level == 0 ? (
                <Loading />
              ) : (
                <div
                  style={{
                    color: `hsl(${toy.battery_level * 120}, 100%, 50%)`,
                  }}
                >
                  {percent.format(toy.battery_level)}
                </div>
              )}
            </div>
            <ToySettingsModal
              isOpen={toySettingsModalIsOpen}
              onClose={() => setToySettingsModalIsOpen(false)}
              toy={toy}
            />
            <div className="grid">
              {toy.features.map((feature) => (
                <Feature
                  toyId={toy.toy_id}
                  feature={feature}
                  key={`${toy.toy_id} ${feature.feature_index}`}
                />
              ))}
            </div>
          </div>
        ))}
      </div>
      <div>
        <button
          type="button"
          onClick={() => toggleScan()}
          className={
            "text-lg font-bold p-1 pl-4 pr-4 m-2  border-gray-500 border-solid border-2 rounded-sm shadow-zinc-900 shadow-md hover:border-gray-300"
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
