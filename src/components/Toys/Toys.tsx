import { useState } from "react";
import { useCoreEventContext } from "../../context/CoreEventContext";
import { useToys } from "../../context/ToysContext";
import { percent } from "../../utils";
import NameBadge from "./NameBadge";
import ToySettingsModal from "./ToySettingsModal";
import { Feature } from "../Feature";

export default function () {
  const [toySettingsModalIsOpen, setToySettingsModalIsOpen] = useState(false);
  const { toys } = useToys();
  const { isScanning, toggleScan } = useCoreEventContext();

  return (
    <div className="flex-col justify-between items-stretch text-lg rounded-lg p-8 m-1 bg-zinc-700 shadow-xl">
      <h1>
        <div className="grad-text grad-backwards text-4xl">Connected toys</div>
      </h1>
      <div className="overflow-y-auto">
        {Object.values(toys).map((toy) => (
          <div
            className="rounded-md bg-zinc-600 shadow-lg p-6"
            key={`${toy.toy_name} ${toy.toy_id}`}
          >
            <div
              className="text-4xl flex justify-between items-center cursor-pointer"
              onClick={() => setToySettingsModalIsOpen(true)}
            >
              <NameBadge name={toy.toy_name} />
              {toy.battery_level == 0 ? (
                <div>...</div>
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
      <div className="flex justify-center">
        <button
          className="btn-custom-2"
          type="button"
          onClick={() => toggleScan()}
        >
          {isScanning ? "Scanning..." : "Search for toys"}
        </button>
      </div>
    </div>
  );
}
