import { useEffect, useState } from "react";
import { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import lovenseLogo from "../assets/Lovense.png";
import lovenseConnectLogo from "../assets/Lovense_Connect.png";
import BatteryIcon from "../components/BatteryIcon";
import Tooltip from "../layout/Tooltip";
import { cn } from "../lib/utils";
import FeatureForm from "./FeatureForm";
import ToySettings from "./ToySettings";

export default function Toy({ toy }: { toy: FeVCToy }) {
  const [selectedFeatureIndex, setSelectedFeatureIndex] = useState(0);
  const nameInfo = NameInfo(toy);

  useEffect(() => {
    if (selectedFeatureIndex >= toy.features.length) setSelectedFeatureIndex(0);
  }, [toy]);

  return (
    <div className="overflow-hidden w-full">
      <div className="text-4xl flex justify-between items-center px-6">
        <div className="flex items-end gap-2">
          <div>{nameInfo.shortName}</div>
          {!toy.toy_connected && (
            <div className="text-sm text-zinc-400">offline</div>
          )}
        </div>
        <ToyInfo nameInfo={nameInfo} battery={toy.battery_level} />
      </div>
      <div className="m-4 overflow-hidden">
        <ToySettings toy={toy} />
        <div className="flex overflow-x-scroll scrollbar select-none w-[calc(100vw-300px)]">
          {toy.features.map((feature, featureArrayIndex) => (
            <button
              key={`${feature.feature_type} ${feature.feature_index}`}
              onClick={() => setSelectedFeatureIndex(featureArrayIndex)}
              className={cn(
                featureArrayIndex == selectedFeatureIndex && "outline",
                feature.feature_enabled ? "text-zinc-200" : "text-zinc-500",
                "rounded-md bg-zinc-700 px-4 py-1 hover:bg-cyan-600 m-2 outline-2 outline-cyan-500 whitespace-nowrap",
              )}
            >
              {feature.feature_type} {feature.feature_index}
            </button>
          ))}
        </div>
        <FeatureForm toy={toy} selectedIndex={selectedFeatureIndex} />
      </div>
    </div>
  );
}

type NameInfo = {
  shortName: string;
  fullName: string;
  logo: string | undefined;
};

function NameInfo(toy: FeVCToy): NameInfo {
  const name = toy.toy_name;

  if (!toy.toy_connected)
    return {
      fullName: name,
      // "Normalized" since Lovense toy_names are saved with whatever first connection method was
      shortName: name.replace("Lovense Connect ", "Lovense "),
      logo: undefined,
    };

  // Shorten everything else since we have the badge
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
