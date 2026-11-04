import BatteryIcon from "@/components/ui/BatteryIcon";
import { useCoreEventContext } from "@/context/CoreEvents";
import Button from "@/layout/Button";
import ThemeIcon from "@/theme/ThemeIcon";
import { FeVCToy } from "@bindings/FeVCToy";
import { FeVCToyFeature } from "@bindings/FeVCToyFeature";
import { ToyPower } from "@bindings/ToyPower";
import { Settings2 } from "lucide-react";
import { useEffect, useState } from "react";
import lovenseLogo from "../assets/Lovense.png";
import Tooltip from "../layout/Tooltip";
import { cn } from "../lib/utils";
import FeatureForm from "./FeatureForm";
import { featureIcons, featureLabel } from "./featureMeta";
import ToySettings from "./ToySettings";

export default function Toy({ toy }: { toy: FeVCToy }) {
  const [selectedFeatureIndex, setSelectedFeatureIndex] = useState(0);
  const nameInfo = NameInfo(toy);

  const { config } = useCoreEventContext();
  // The Config "advanced" switches now decide what starts open
  const [toySettingsOpen, setToySettingsOpen] = useState(
    () => config?.show_toy_advanced ?? false,
  );
  const [advancedOpen, setAdvancedOpen] = useState(
    () => config?.show_feature_advanced ?? false,
  );

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
    <div className="flex h-full w-full min-w-0 flex-col">
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-3 px-2 pb-2">
        <div className="flex min-w-0 items-center gap-3 text-4xl font-bold">
          <ThemeIcon className="h-8 w-8 shrink-0 text-theme-primary" />
          <span className="truncate bg-gradient-to-r from-theme-primary via-theme-secondary to-theme-info bg-clip-text text-transparent">
            {nameInfo.shortName}
          </span>
        </div>
        <div className="flex items-center gap-4">
          <Button
            variant="secondary"
            onClick={() => setToySettingsOpen((o) => !o)}
            aria-expanded={toySettingsOpen}
            className={cn(
              "m-0 flex items-center gap-1.5 px-3 py-0.5 text-sm",
              toySettingsOpen &&
                "border-theme-primary bg-theme-primary/15 text-theme-primary",
            )}
          >
            <Settings2 className="h-4 w-4" />
            Toy settings
          </Button>
          <ToyInfo nameInfo={nameInfo} toyPower={toy.toy_power} />
        </div>
      </div>

      <div className="flex min-h-0 min-w-0 flex-1 flex-col px-2">
        {toySettingsOpen && (
          <div className="shrink-0">
            <ToySettings toy={toy} />
          </div>
        )}

        <div className="scrollbar flex w-full min-w-0 shrink-0 select-none gap-3 overflow-x-auto pb-2 pt-3">
          {toy.features.map((feature, featureArrayIndex) => (
            <FeatureChip
              key={toyFeatureKey(toy, feature)}
              feature={feature}
              label={featureLabel(toy.features, featureArrayIndex)}
              selected={featureArrayIndex == clampedIndex}
              onClick={() => setSelectedFeatureIndex(featureArrayIndex)}
            />
          ))}
        </div>

        {toy.features.length > 0 ? (
          <FeatureForm
            toy={toy}
            selectedIndex={clampedIndex}
            label={featureLabel(toy.features, clampedIndex)}
            advancedOpen={advancedOpen}
            onAdvancedOpenChange={setAdvancedOpen}
            key={toyFeatureKey(toy, toy.features[clampedIndex])}
          />
        ) : (
          <div className="mt-4 text-sm text-theme-muted">
            No features available for this toy.
          </div>
        )}
      </div>
    </div>
  );
}

/** Feature tab showing its type, on/off state and parameter count */
function FeatureChip({
  feature,
  label,
  selected,
  onClick,
}: {
  feature: FeVCToyFeature;
  label: string;
  selected: boolean;
  onClick: () => void;
}) {
  const Icon = featureIcons[feature.feature_type];
  const paramCount = feature.osc_parameters.length;
  return (
    <button
      onClick={onClick}
      aria-pressed={selected}
      className={cn(
        "flex shrink-0 flex-col items-start gap-0.5 rounded-2xl border-2 px-3 py-1.5 text-left transition-all duration-200",
        selected
          ? "border-theme-primary bg-theme-primary/15"
          : "border-transparent bg-theme-raised hover:border-theme-border",
      )}
    >
      <span
        className={cn(
          "flex items-center gap-1.5 whitespace-nowrap text-sm font-semibold",
          feature.feature_enabled ? "text-theme-text" : "text-theme-muted",
        )}
      >
        <Icon className="h-4 w-4 text-theme-secondary" />
        {label}
      </span>
      <span className="flex items-center gap-1.5 whitespace-nowrap text-xs text-theme-muted">
        <span
          className={cn(
            "h-2 w-2 rounded-full",
            feature.feature_enabled ? "bg-theme-success" : "bg-theme-muted/50",
          )}
        />
        {feature.feature_enabled ? "On" : "Off"} · {paramCount}{" "}
        {paramCount === 1 ? "param" : "params"}
      </span>
    </button>
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
