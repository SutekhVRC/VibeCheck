import PageHeader from "@/components/PageHeader";
import BatteryIcon from "@/components/ui/BatteryIcon";
import { useCoreEventContext } from "@/context/CoreEvents";
import Button from "@/layout/Button";
import ThemeIcon from "@/theme/ThemeIcon";
import type { FeToyEvent } from "@bindings/FeToyEvent";
import { FeVCToy } from "@bindings/FeVCToy";
import { FeVCToyFeature } from "@bindings/FeVCToyFeature";
import { ToyPower } from "@bindings/ToyPower";
import { listen } from "@tauri-apps/api/event";
import { Settings2, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import lovenseLogo from "../assets/Lovense.png";
import { LISTEN } from "../data/constants";
import { appExtensions } from "../extensions";
import Tooltip from "../layout/Tooltip";
import { cn } from "../lib/utils";
import FeatureForm from "./FeatureForm";
import { featureIcons, featureLabel } from "./featureMeta";
import ToySettings from "./ToySettings";

/** How long the remove button waits for the confirming second click */
const CONFIRM_TIMEOUT_MS = 4000;

export default function Toy({
  toy,
  onRemove,
}: {
  toy: FeVCToy;
  onRemove: (toy: FeVCToy) => Promise<void>;
}) {
  const [selectedFeatureIndex, setSelectedFeatureIndex] = useState(0);
  const nameInfo = NameInfo(toy);
  const ToyHeaderSlot = appExtensions.ToyHeader;
  // The forms keep their own copy of the settings while open: when the
  // backend loads them again (e.g. after an avatar change), start the forms
  // over with the new ones
  const [reloads, setReloads] = useState(0);
  useEffect(() => {
    const unlisten = listen<FeToyEvent>(LISTEN.TOY_EVENT, (event) => {
      if (event.payload.kind === "Reloaded") setReloads((n) => n + 1);
    });
    return () => {
      unlisten.then((u) => u());
    };
  }, []);

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
      <PageHeader
        className="px-2 pb-2"
        icon={<ThemeIcon className="h-5 w-5" />}
        title={nameInfo.shortName}
        subtitle={`${toy.toy_connected ? "Connected" : "Offline"} · ${
          toy.features.length
        } ${toy.features.length === 1 ? "feature" : "features"}`}
        actions={
          <div className="flex items-center gap-4">
            {ToyHeaderSlot && <ToyHeaderSlot toy={toy} />}
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
            {!toy.toy_connected && (
              <RemoveToyButton
                key={toy.toy_name}
                toy={toy}
                onRemove={onRemove}
              />
            )}
            <ToyInfo nameInfo={nameInfo} toyPower={toy.toy_power} />
          </div>
        }
      />

      <div className="flex min-h-0 min-w-0 flex-1 flex-col px-2">
        {toySettingsOpen && (
          <div className="shrink-0">
            <ToySettings toy={toy} key={reloads} />
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
            key={`${toyFeatureKey(toy, toy.features[clampedIndex])} ${reloads}`}
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

/** Forgets an offline toy. A trash icon that opens to "Delete Toy?" on the
 * first click; the second click deletes. */
function RemoveToyButton({
  toy,
  onRemove,
}: {
  toy: FeVCToy;
  onRemove: (toy: FeVCToy) => Promise<void>;
}) {
  const [confirming, setConfirming] = useState(false);
  const [removing, setRemoving] = useState(false);

  useEffect(() => {
    if (!confirming) return;
    const t = setTimeout(() => setConfirming(false), CONFIRM_TIMEOUT_MS);
    return () => clearTimeout(t);
  }, [confirming]);

  async function handleClick() {
    if (!confirming) {
      setConfirming(true);
      return;
    }
    setConfirming(false);
    setRemoving(true);
    await onRemove(toy);
    setRemoving(false);
  }

  return (
    <Tooltip text="Forget this toy and delete its saved settings. It comes back with default settings if it connects again.">
      <Button
        variant="danger"
        onClick={handleClick}
        disabled={removing}
        aria-label={confirming ? "Delete toy? Click to confirm" : "Delete toy"}
        className={cn(
          "m-0 flex items-center py-1 text-sm",
          confirming ? "gap-1.5 px-3" : "px-1.5",
        )}
      >
        <Trash2 className="h-4 w-4 shrink-0" />
        {/* The label slides open on the first click */}
        <span
          className={cn(
            "overflow-hidden whitespace-nowrap transition-[max-width] duration-200",
            confirming ? "max-w-[7rem]" : "max-w-0",
          )}
        >
          Delete Toy?
        </span>
      </Button>
    </Tooltip>
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
