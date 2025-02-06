import { useCoreEventContext } from "@/context/CoreEvents";
import { useEffect, useState } from "react";
import type { FeVCToyFeature } from "src-tauri/bindings/FeVCToyFeature";
import type { ToyPower } from "src-tauri/bindings/ToyPower";
import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
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

  const { config } = useCoreEventContext();

  useEffect(() => {
    // update from external - because toy could go online/offline and be 'different'
    if (selectedFeatureIndex >= toy.features.length) setSelectedFeatureIndex(0);
  }, [toy]);

  function toyFeatureKey(t: FeVCToy, f: FeVCToyFeature) {
    return `${t.toy_name} ${t.sub_id} ${f.feature_type} ${f.feature_index}`;
  }

  return (
    <div className="h-full w-full">
      <div className="flex items-center justify-between px-2 text-4xl">
        <div>{nameInfo.shortName}</div>
        <ToyInfo nameInfo={nameInfo} toyPower={toy.toy_power} />
      </div>
      <div className="px-2">
        {config?.show_toy_advanced && <ToySettings toy={toy} />}
        {/* pt-2 because scrollbar gap is bottom */}
        <div className="scrollbar flex w-[calc(100vw-320px)] select-none  items-center gap-4 overflow-x-scroll pt-2 md:w-[calc(100vw-340px)]">
          {toy.features.map((feature, featureArrayIndex) => (
            <button
              key={toyFeatureKey(toy, feature)}
              onClick={() => setSelectedFeatureIndex(featureArrayIndex)}
              className={cn(
                "whitespace-nowrap rounded-md border-2 border-transparent bg-zinc-700 px-4 py-1 hover:bg-cyan-600",
                feature.feature_enabled ? "text-zinc-200" : "text-zinc-500",
                featureArrayIndex == selectedFeatureIndex && "border-cyan-500",
              )}
            >
              {feature.feature_type} {feature.feature_index}
            </button>
          ))}
        </div>
        <FeatureForm
          toy={toy}
          selectedIndex={selectedFeatureIndex}
          key={toyFeatureKey(toy, toy.features[selectedFeatureIndex])}
        />
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
  toyPower,
}: {
  nameInfo: NameInfo;
  toyPower: ToyPower;
}) {
  return (
    <div className="flex items-center gap-x-4">
      {nameInfo.logo != undefined && (
        <Tooltip text={nameInfo.fullName}>
          <img className="max-h-6 cursor-help rounded-lg" src={nameInfo.logo} />
        </Tooltip>
      )}
      <BatteryIcon toyPower={toyPower} />
    </div>
  );
}
