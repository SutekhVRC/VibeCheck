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
    <div className="flex-col justify-between items-stretch text-lg rounded-lg p-4 bg-zinc-800">
      <h1>
        <div className="grad-backwards text-clip text-4xl">Connected toys</div>
      </h1>
      <div
        className="overflow-y-scroll pl-2 scrollbar pt-2 pb-2"
        style={{ maxHeight: "calc(100vh - 420px)" }}
      >
        {Object.values(toys).map((toy) => (
          <div
            className="rounded-md bg-zinc-700 p-4 m-2"
            key={`${toy.toy_name} ${toy.toy_id}`}
          >
            <div
              className="text-4xl flex justify-between items-center cursor-pointer"
              onClick={() => setToySettingsModalIsOpen(true)}
            >
              <NameBadge name={toy.toy_name} />
              {toy.battery_level == 0 ? (
                <div className="flex">
                  <Loading />%
                </div>
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
                <>
                  <hr className="border-1 border-zinc-800 m-1 border-opacity-75" />
                  <Feature
                    toyId={toy.toy_id}
                    feature={feature}
                    key={`${toy.toy_id} ${feature.feature_type} ${feature.feature_index}`}
                  />
                </>
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
