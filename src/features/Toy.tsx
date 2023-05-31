import { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import lovenseLogo from "../assets/Lovense.png";
import lovenseConnectLogo from "../assets/Lovense_Connect.png";
import FeatureForm from "./FeatureForm";
import ToySettings from "./ToySettings";
import Tooltip from "../layout/Tooltip";
import BatteryIcon from "../components/BatteryIcon";
import { ALTER_TOY } from "../data/constants";
import { invoke } from "@tauri-apps/api";
import { useToastContext } from "../context/ToastContext";
import { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";
import classNames from "classnames";
import { useState } from "react";

export default function Toy({ toy }: { toy: FeVCToy }) {
  const [selectedFeature, setSelectedFeature] = useState<FeVCToyFeature | null>(
    toy.features[0]
  );
  const nameInfo = NameInfo(toy.toy_name);
  const toast = useToastContext();

  async function handleToyAlter(newToy: FeVCToy) {
    try {
      if (newToy.toy_connected) {
        await invoke(ALTER_TOY, {
          mutate: { Connected: newToy },
        });
      } else {
        await invoke(ALTER_TOY, {
          mutate: { Disconnected: newToy },
        });
      }
    } catch (e) {
      toast.createToast("Could not alter toy!", JSON.stringify(e), "error");
    }
  }

  function handleFeatureAlter(newFeature: FeVCToyFeature) {
    toy.features[newFeature.feature_index] = newFeature;
    handleToyAlter(toy);
  }

  return (
    <div className="w-full">
      <div className="text-4xl flex justify-between items-center px-6">
        <div className="flex items-end gap-2">
          <div>{nameInfo.shortName}</div>
          {!toy.toy_connected && (
            <div className="text-sm text-slate-400">offline</div>
          )}
        </div>
        <ToyInfo nameInfo={nameInfo} battery={toy.battery_level} />
      </div>
      <ToySettings toy={toy} handleToyAlter={handleToyAlter} />
      <div className="flex">
        {toy.features.map((feature) => (
          <button
            key={`${feature.feature_type} ${feature.feature_index}`}
            onClick={() => setSelectedFeature(feature)}
            className={classNames(
              feature.feature_type == selectedFeature?.feature_type &&
                feature.feature_index == selectedFeature.feature_index &&
                "outline",
              "rounded-md bg-gray-700 px-4 py-1 hover:bg-cyan-600 m-2 outline-2 outline-emerald-500"
            )}
          >
            {feature.feature_type} {feature.feature_index}
          </button>
        ))}
      </div>
      {selectedFeature != null && (
        <FeatureForm
          key={`${selectedFeature.feature_type} ${selectedFeature.feature_index}`}
          handleFeatureAlter={handleFeatureAlter}
          toyId={toy.toy_id}
          oldFeature={selectedFeature}
        />
      )}
    </div>
  );
}

type NameInfo = {
  shortName: string;
  fullName: string;
  logo: string | undefined;
};

function NameInfo(name: string): NameInfo {
  if (name.startsWith("Lovense Connect")) {
    return {
      fullName: name,
      shortName: name.replace("Lovense Connect ", ""),
      logo: lovenseConnectLogo,
    };
  } else if (name.startsWith("Lovense")) {
    return {
      fullName: name,
      shortName: name.replace("Lovense ", ""),
      logo: lovenseLogo,
    };
  }
  return {
    shortName: name,
    fullName: name,
    logo: undefined,
  };
}

function ToyInfo({
  nameInfo,
  battery,
}: {
  nameInfo: NameInfo;
  battery: number | null;
}) {
  return (
    <div className="flex gap-x-4 items-center">
      {nameInfo.logo != undefined && (
        <Tooltip text={nameInfo.fullName}>
          <img className="max-h-6 rounded-lg cursor-help" src={nameInfo.logo} />
        </Tooltip>
      )}
      <BatteryIcon battery={battery} />
    </div>
  );
}
