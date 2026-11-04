import { useCoreEventContext } from "@/context/CoreEvents";
import { FeVCToy } from "@bindings/FeVCToy";
import { FeVCToyFeature } from "@bindings/FeVCToyFeature";
import { ToyPower } from "@bindings/ToyPower";
import { PawPrint } from "lucide-react";
import { useEffect, useState } from "react";
import lovenseLogo from "../assets/Lovense.png";
import BatteryIcon from "../components/BatteryIcon";
import Tooltip from "../layout/Tooltip";
import { cn } from "../lib/utils";
import FeatureForm from "./FeatureForm";
import ToySettings from "./ToySettings";

export default function Toy({ toy }: { toy: FeVCToy }) {
  const [selectedFeatureIndex, setSelectedFeatureIndex] = useState(0);
  const nameInfo = NameInfo(toy);

  const { config } = useCoreEventContext();
  const clampedIndex =
    toy.features.length === 0
      ? 0
      : Math.min(selectedFeatureIndex, toy.features.length - 1);

  useEffect(() => {
    // update from external - because toy could go online/offline and be 'different'
    if (selectedFeatureIndex >= toy.features.length) setSelectedFeatureIndex(0);
  }, [toy, selectedFeatureIndex]);

  function toyFeatureKey(t: FeVCToy, f: FeVCToyFeature) {
    return `${t.toy_name} ${t.sub_id} ${f.feature_type} ${f.feature_index}`;
  }

  return (
    <div className="h-full w-full">
      <div className="flex items-center justify-between px-2 pb-2 text-4xl">
        <div className="flex items-center gap-3 font-bold">
          <PawPrint className="h-8 w-8 text-cute-pink" />
          <span className="bg-gradient-to-r from-cute-pink via-cute-lilac to-cute-sky bg-clip-text text-transparent">
            {nameInfo.shortName}
          </span>
        </div>
        <ToyInfo nameInfo={nameInfo} toyPower={toy.toy_power} />
      </div>
      <div className="px-2">
        {config?.show_toy_advanced && <ToySettings toy={toy} />}
        {/* pt-2 because scrollbar gap is bottom */}
        <div className="scrollbar flex w-[calc(100vw-320px)] select-none items-center gap-4 overflow-x-scroll pt-2 md:w-[calc(100vw-340px)]">
          {toy.features.map((feature, featureArrayIndex) => (
            <button
              key={toyFeatureKey(toy, feature)}
              onClick={() => setSelectedFeatureIndex(featureArrayIndex)}
              className={cn(
                "whitespace-nowrap rounded-full border-2 px-4 py-1 transition-all duration-200",
                feature.feature_enabled ? "text-cute-cream" : "text-cute-muted",
                featureArrayIndex == selectedFeatureIndex
                  ? "border-cute-pink bg-cute-pink/20"
                  : "border-transparent bg-cute-raised hover:bg-cute-line",
              )}
            >
              {feature.feature_type} {feature.feature_index}
            </button>
          ))}
        </div>
        {toy.features.length > 0 ? (
          <FeatureForm
            toy={toy}
            selectedIndex={clampedIndex}
            key={toyFeatureKey(toy, toy.features[clampedIndex])}
          />
        ) : (
          <div className="mt-4 text-sm text-cute-muted">
            No features available for this toy.
          </div>
        )}
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

  // Shorten everything else since we have the badge
  if (name.startsWith("Lovense")) {
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
