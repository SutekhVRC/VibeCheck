import lovenseLogo from "../assets/Lovense.png";
import lovenseConnectLogo from "../assets/Lovense_Connect.png";
import FeatureForm from "./FeatureForm";
import ToySettings from "./ToySettings";
import Tooltip from "../layout/Tooltip";
import BatteryIcon from "../components/BatteryIcon";
import { useEffect, useState } from "react";
import { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import { cn } from "../utils";

export default function Toy({ toy }: { toy: FeVCToy }) {
  const [selectedFeatureIndex, setSelectedFeatureIndex] = useState(0);
  const nameInfo = NameInfo(toy.toy_name);

  useEffect(() => {
    if (selectedFeatureIndex >= toy.features.length) setSelectedFeatureIndex(0);
  }, [toy]);

  return (
    <div className="w-fit overflow-hidden">
      <div className="text-4xl flex justify-between items-center px-6">
        <div className="flex items-end gap-2">
          <div>{nameInfo.shortName}</div>
          {!toy.toy_connected && (
            <div className="text-sm text-slate-400">offline</div>
          )}
        </div>
        <ToyInfo nameInfo={nameInfo} battery={toy.battery_level} />
      </div>
      <div className="m-4 overflow-hidden">
        <ToySettings toy={toy} />
        <div className="flex overflow-x-scroll scrollbar select-none">
          {toy.features.map((feature, featureArrayIndex) => (
            <button
              key={`${feature.feature_type} ${feature.feature_index}`}
              onClick={() => setSelectedFeatureIndex(featureArrayIndex)}
              className={cn(
                featureArrayIndex == selectedFeatureIndex && "outline",
                feature.feature_enabled ? "text-gray-200" : "text-gray-500",
                "rounded-md bg-gray-700 px-4 py-1 hover:bg-cyan-600 m-2 outline-2 outline-cyan-500 whitespace-nowrap"
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
