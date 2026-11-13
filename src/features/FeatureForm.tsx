import { SettingRow, SettingsGroup } from "@/components/SettingRow";
import { FreeTextOptions } from "@/components/ui/FreeTextOptions";
import { PenetrationSystems, ProcessingModes } from "@/data/stringArrayTypes";
import Button from "@/layout/Button";
import { Select } from "@/layout/Select";
import Tooltip from "@/layout/Tooltip";
import { cn } from "@/lib/utils";
import { FeLevelTweaks } from "@bindings/FeLevelTweaks";
import { FeProcessingMode } from "@bindings/FeProcessingMode";
import { FeToyParameter } from "@bindings/FeToyParameter";
import { FeVCToy } from "@bindings/FeVCToy";
import type { FeVCToyFeature } from "@bindings/FeVCToyFeature";
import { ScrollArea } from "@radix-ui/react-scroll-area";
import { DebouncedFunc, debounce } from "lodash";
import { ChevronRight, Plus, X } from "lucide-react";
import {
  ChangeEvent,
  Dispatch,
  ReactNode,
  SetStateAction,
  createContext,
  useCallback,
  useContext,
  useState,
} from "react";
import EditableNumber from "../components/EditableNumber";
import { OSC, TOOLTIP } from "../data/constants";
import { handleFeatureAlter as handleToyFeatureAlter } from "../hooks/useToys";
import Slider from "../layout/Slider";
import Switch from "../layout/Switch";
import { featureIcons } from "./featureMeta";
import FeatureTest from "./FeatureTest";

type FeatureFormContextProps = {
  feature: FeVCToyFeature;
  setToyFeature: Dispatch<SetStateAction<FeVCToyFeature>>;
  debouncedAlter: DebouncedFunc<(f: FeVCToyFeature) => void>;
  handleFeatureAlter: (f: FeVCToyFeature) => void;
  handleBool: (checked: boolean, name: keyof FeVCToyFeature) => void;
  handleLevels: (key: keyof FeLevelTweaks, value: number) => void;
  commitLevels: (levels: Partial<FeLevelTweaks>) => void;
};

const FeatureFormContext = createContext<FeatureFormContextProps>({
  feature: {} as FeVCToyFeature,
  setToyFeature: () => null,
  debouncedAlter: debounce(() => null, 1000),
  handleFeatureAlter: () => null,
  handleBool: () => null,
  handleLevels: () => null,
  commitLevels: () => null,
});

const useFeatureFormContext = () => {
  const context = useContext(FeatureFormContext);
  if (!context) {
    throw new Error("useFeatureFormContext not within context provider");
  }
  return context;
};

type ToyFeatureFormProps = {
  toy: FeVCToy;
  selectedIndex: number;
  /** Friendly feature name, e.g. "Vibrator 1" */
  label: string;
  /** Whether the Advanced section is expanded (kept by the toy page) */
  advancedOpen: boolean;
  onAdvancedOpenChange: (open: boolean) => void;
};

export default function FeatureForm({
  toy,
  selectedIndex,
  label,
  advancedOpen,
  onAdvancedOpenChange,
}: ToyFeatureFormProps) {
  const [feature, setToyFeature] = useState(
    toy.features[selectedIndex] ?? toy.features[0],
  );

  // Only need debounce for input fields, levels work with onValueCommit
  // Fast debounce because otherwise we'd have to merge with other updates
  const debouncedAlter = useCallback(
    debounce((f) => handleFeatureAlter(f), 100),
    [],
  );

  function handleFeatureAlter(feature: FeVCToyFeature) {
    handleToyFeatureAlter(toy, feature);
  }

  function handleBool(checked: boolean, name: keyof FeVCToyFeature) {
    setToyFeature((f) => {
      const newF = { ...f, [name]: checked } as FeVCToyFeature;
      handleFeatureAlter(newF);
      return newF;
    });
  }

  function handleLevels(key: keyof FeLevelTweaks, value: number) {
    setToyFeature((feature) => {
      return {
        ...feature,
        feature_levels: { ...feature.feature_levels, [key]: value },
      };
    });
  }

  // Set levels and send them to the backend in one go (used by typed values)
  function commitLevels(newLevels: Partial<FeLevelTweaks>) {
    setToyFeature((f) => {
      const newF = {
        ...f,
        feature_levels: { ...f.feature_levels, ...newLevels },
      };
      handleFeatureAlter(newF);
      return newF;
    });
  }

  const tweakSliders = feature.osc_parameters.reduce(
    (seenModes, oscParam) => {
      seenModes.add(oscParam.processing_mode);
      return seenModes;
    },
    new Set<string>([feature.penetration_system.pen_system_processing_mode]),
  );

  const Icon = featureIcons[feature.feature_type];

  return (
    <FeatureFormContext.Provider
      value={{
        feature,
        setToyFeature,
        debouncedAlter,
        handleFeatureAlter,
        handleBool,
        handleLevels,
        commitLevels,
      }}
    >
      <div className="mt-1 flex min-h-0 flex-1 flex-col rounded-2xl border border-theme-border/60 bg-theme-raised">
        {/* Header: the two things you reach for most */}
        <div className="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b border-theme-border/40 px-4 py-2.5">
          <div className="flex items-center gap-2 text-base font-semibold">
            <Icon className="h-5 w-5 text-theme-secondary" />
            {label}
          </div>
          <div className="flex items-center gap-3">
            <Tooltip text={TOOLTIP.Enabled.text}>
              <label className="flex cursor-pointer items-center gap-2 text-sm">
                <span
                  className={
                    feature.feature_enabled
                      ? "text-theme-text"
                      : "text-theme-muted"
                  }
                >
                  {feature.feature_enabled ? "Enabled" : "Disabled"}
                </span>
                <Switch
                  checked={feature.feature_enabled}
                  onCheckedChange={(checked) =>
                    handleBool(checked, "feature_enabled")
                  }
                />
              </label>
            </Tooltip>
            <FeatureTest toy={toy} feature={feature} label={label} />
          </div>
        </div>

        <FeatureScrollArea>
          <div className="flex flex-col gap-4 px-4 pb-4 pt-2">
            <SettingsGroup title="Input">
              <Parameters />
              <InputProcessor />
              <InputFilter />
            </SettingsGroup>

            <SettingsGroup title="Output">
              <Range />
            </SettingsGroup>

            <div className="text-left text-sm">
              <button
                type="button"
                onClick={() => onAdvancedOpenChange(!advancedOpen)}
                aria-expanded={advancedOpen}
                className="group flex w-full items-center gap-2 rounded-lg py-1 text-left"
              >
                <ChevronRight
                  className={cn(
                    "h-4 w-4 text-theme-muted transition-transform duration-200",
                    advancedOpen && "rotate-90",
                  )}
                />
                <span className="text-xs font-semibold uppercase tracking-wider text-theme-muted group-hover:text-theme-secondary">
                  Advanced
                </span>
                {!advancedOpen && (
                  <span className="truncate text-xs text-theme-muted/70">
                    Idle speed, flip input, smoothing…
                  </span>
                )}
              </button>
              {advancedOpen && (
                <SettingsGroup>
                  <Idle />
                  <FlipInput />
                  {feature.feature_type == "Linear" && <Linear />}
                  {tweakSliders.has("Smooth") && <Smooth />}
                  {tweakSliders.has("Rate") && <Rate />}
                  {tweakSliders.has("Constant") && <Constant />}
                </SettingsGroup>
              )}
            </div>
          </div>
        </FeatureScrollArea>
      </div>
    </FeatureFormContext.Provider>
  );
}

function InputProcessor() {
  const { feature, setToyFeature, handleFeatureAlter } =
    useFeatureFormContext();

  function handleInputProcessor<
    K extends "pen_system_type" | "pen_system_processing_mode",
  >(key: K, value: FeVCToyFeature["penetration_system"][K]) {
    setToyFeature((f) => {
      const newF = {
        ...f,
        penetration_system: {
          ...f.penetration_system,
          [key]: value,
        },
      };
      handleFeatureAlter(newF);
      return newF;
    });
  }

  return (
    <SettingRow label="Processor" tooltip={TOOLTIP.InputProcessor}>
      <div className="grid grid-cols-2 gap-3">
        <label className="flex flex-col gap-1">
          <span className="text-xs text-theme-muted">System</span>
          <Select
            name="pen_system_type"
            value={feature.penetration_system.pen_system_type}
            onValueChange={(v) => handleInputProcessor("pen_system_type", v)}
            options={PenetrationSystems}
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-xs text-theme-muted">Mode</span>
          <Select
            name="pen_system_processing_mode"
            value={feature.penetration_system.pen_system_processing_mode}
            onValueChange={(v) =>
              handleInputProcessor("pen_system_processing_mode", v)
            }
            options={ProcessingModes}
          />
        </label>
      </div>
    </SettingRow>
  );
}

function Parameters() {
  const { feature, setToyFeature, handleFeatureAlter, debouncedAlter } =
    useFeatureFormContext();

  function removeParam(parameter: string) {
    setToyFeature((f) => {
      const newF = {
        ...f,
        osc_parameters: f.osc_parameters.filter(
          (param) => param.parameter != parameter,
        ),
      };
      handleFeatureAlter(newF);
      return newF;
    });
  }

  function findParamName(params: FeToyParameter[]) {
    // A bit overpowered for need but :shrug:
    const seenSuffixes = params.reduce((acc, val) => {
      const [_, suffix] = val.parameter.split("param-").map((c) => parseInt(c));
      if (isNaN(suffix)) return acc;
      acc.add(suffix);
      return acc;
    }, new Set<number>());
    for (let i = 0; i < params.length; i++) if (!seenSuffixes.has(i)) return i;
    return params.length;
  }

  function addParam() {
    setToyFeature((f) => {
      const newParam = `${OSC.PARAM_PREFIX}param-${findParamName(
        f.osc_parameters,
      )}`;
      const newF = {
        ...f,
        osc_parameters: [
          ...f.osc_parameters,
          {
            parameter: newParam,
            processing_mode: "Raw" as const,
          },
        ],
      };
      handleFeatureAlter(newF);
      return newF;
    });
  }

  function handleOscParam(
    e: ChangeEvent<HTMLInputElement>,
    paramIndex: number,
  ) {
    setToyFeature((f) => {
      const newParams = [...f.osc_parameters];
      newParams[paramIndex] = {
        ...newParams[paramIndex],
        parameter: normalizeOscParameter(e.target.value),
      };
      const newF = {
        ...f,
        osc_parameters: newParams,
      };
      debouncedAlter(newF);
      return newF;
    });
  }

  function handleOscParamMode(mode: FeProcessingMode, paramIndex: number) {
    setToyFeature((f) => {
      const newParams = [...f.osc_parameters];
      newParams[paramIndex] = {
        ...newParams[paramIndex],
        processing_mode: mode,
      };
      const newF = {
        ...f,
        osc_parameters: newParams,
      };
      handleFeatureAlter(newF);
      return newF;
    });
  }

  function normalizeOscParameter(p: string) {
    return `${OSC.PARAM_PREFIX}${p.replaceAll(" ", "_")}`;
  }

  return (
    <SettingRow label="Parameters">
      <div className="flex flex-col gap-2">
        {feature.osc_parameters.length === 0 && (
          <div className="rounded-xl border border-dashed border-theme-border px-3 py-2 text-xs text-theme-muted">
            No parameters yet. Add one so VRChat can drive this feature.
          </div>
        )}
        {feature.osc_parameters.map((param, paramIndex) => (
          // Keyed by index on purpose: keying by the parameter name would
          // remount the input on every keystroke and lose focus while typing
          <div
            key={paramIndex}
            className="grid grid-cols-[minmax(0,1fr)_8rem_auto] items-center gap-2"
          >
            <div className="theme-input flex min-w-0 items-center focus-within:border-theme-primary focus-within:ring-2 focus-within:ring-theme-primary/30">
              <span className="shrink-0 select-none text-xs text-theme-muted">
                {OSC.PARAM_PREFIX}
              </span>
              <input
                className="min-w-0 flex-1 bg-transparent outline-none"
                name="osc_parameter"
                aria-label="OSC parameter name"
                value={param.parameter.replace(OSC.PARAM_PREFIX, "")}
                onChange={(e) => handleOscParam(e, paramIndex)}
              />
            </div>
            <Select
              name="osc_parameter_mode"
              value={param.processing_mode}
              onValueChange={(mode) => handleOscParamMode(mode, paramIndex)}
              options={ProcessingModes}
            />
            <Tooltip text="Remove parameter">
              <button
                type="button"
                aria-label="Remove parameter"
                className="flex h-7 w-7 items-center justify-center rounded-full text-theme-muted transition-colors hover:bg-theme-danger/15 hover:text-theme-danger"
                onClick={() => removeParam(param.parameter)}
              >
                <X className="h-4 w-4" />
              </button>
            </Tooltip>
          </div>
        ))}
        <div>
          <Button
            type="button"
            variant="secondary"
            onClick={addParam}
            className="m-0 flex items-center gap-1.5 px-3 py-0.5 text-sm"
          >
            <Plus className="h-4 w-4" />
            Add parameter
          </Button>
        </div>
      </div>
    </SettingRow>
  );
}

function InputFilter() {
  const { feature, setToyFeature, handleFeatureAlter } =
    useFeatureFormContext();

  function handleInputProcessor(items: string[]) {
    setToyFeature((f) => {
      const newF: FeVCToyFeature = {
        ...f,
        penetration_system: {
          ...f.penetration_system,
          pen_system_input_filter: items,
        },
      };
      handleFeatureAlter(newF);
      return newF;
    });
  }
  return (
    <SettingRow label="Input Filter" tooltip={TOOLTIP.InputFilter}>
      <FreeTextOptions
        values={feature.penetration_system.pen_system_input_filter ?? []}
        onChange={handleInputProcessor}
        placeholder="Add Filter Option"
        transform={(s) => s.replaceAll(" ", "_")}
        validator={{
          // eslint-disable-next-line no-useless-escape
          re: /^[\w\/]+$/,
          message: "Only alphanumeric characters and slashes allowed",
        }}
      />
    </SettingRow>
  );
}

function Linear() {
  const { feature, handleLevels, handleFeatureAlter, commitLevels } =
    useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <SettingRow
      label="Linear Speed"
      tooltip={TOOLTIP.LinearSpeed}
      control={
        <EditableNumber
          value={levels.linear_position_speed}
          min={10}
          max={1000}
          step={1}
          onCommit={(v) => commitLevels({ linear_position_speed: v })}
          label="Linear speed"
        />
      }
    >
      <Slider
        min={10}
        max={1000}
        step={1}
        value={[levels.linear_position_speed]}
        onValueChange={(e) => handleLevels("linear_position_speed", e[0])}
        onValueCommit={() => handleFeatureAlter(feature)}
      />
    </SettingRow>
  );
}

function Idle() {
  const { feature, handleLevels, handleFeatureAlter, commitLevels } =
    useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <SettingRow
      label="Idle Speed"
      tooltip={TOOLTIP.Idle}
      flipped={feature.flip_input_float}
      control={
        <EditableNumber
          value={levels.idle_level}
          min={0}
          max={1}
          step={0.01}
          scale={100}
          onCommit={(v) => commitLevels({ idle_level: v })}
          label="Idle speed"
        />
      }
    >
      <Slider
        multiply={100}
        min={0}
        max={1}
        step={0.01}
        value={[levels.idle_level]}
        onValueChange={(e) => handleLevels("idle_level", e[0])}
        onValueCommit={() => handleFeatureAlter(feature)}
      />
    </SettingRow>
  );
}

function Range() {
  const { feature, setToyFeature, handleFeatureAlter, commitLevels } =
    useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <SettingRow
      label="Range"
      tooltip={TOOLTIP.Range}
      flipped={feature.flip_input_float}
      control={
        <>
          <EditableNumber
            value={levels.minimum_level}
            min={0}
            max={levels.maximum_level}
            step={0.01}
            scale={100}
            onCommit={(v) => commitLevels({ minimum_level: v })}
            label="Range minimum"
          />
          <span className="text-theme-muted">–</span>
          <EditableNumber
            value={levels.maximum_level}
            min={levels.minimum_level}
            max={1}
            step={0.01}
            scale={100}
            onCommit={(v) => commitLevels({ maximum_level: v })}
            label="Range maximum"
          />
        </>
      }
    >
      <Slider
        multiply={100}
        min={0}
        max={1}
        step={0.01}
        value={[levels.minimum_level, levels.maximum_level]}
        onValueChange={(e) => {
          setToyFeature((f) => {
            return {
              ...f,
              feature_levels: {
                ...f.feature_levels,
                minimum_level: e[0],
                maximum_level: e[1],
              },
            };
          });
        }}
        onValueCommit={() => handleFeatureAlter(feature)}
      />
    </SettingRow>
  );
}

function FlipInput() {
  const { feature, handleBool } = useFeatureFormContext();
  return (
    <SettingRow
      label="Flip Input"
      tooltip={TOOLTIP.FlipInput}
      control={
        <Switch
          size="small"
          checked={feature.flip_input_float}
          onCheckedChange={(checked) => handleBool(checked, "flip_input_float")}
        />
      }
    />
  );
}

function Smooth() {
  const { feature, handleLevels, handleFeatureAlter, commitLevels } =
    useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <SettingRow
      label="Smooth Level"
      tooltip={TOOLTIP.Smooth}
      control={
        <EditableNumber
          value={levels.smooth_rate}
          min={1}
          max={20}
          step={1}
          onCommit={(v) => commitLevels({ smooth_rate: v })}
          label="Smooth level"
        />
      }
    >
      <Slider
        min={1}
        max={20}
        step={1}
        value={[levels.smooth_rate]}
        onValueChange={(e) => handleLevels("smooth_rate", e[0])}
        onValueCommit={() => handleFeatureAlter(feature)}
      />
    </SettingRow>
  );
}

function Rate() {
  const { feature, handleLevels, handleFeatureAlter, commitLevels } =
    useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <>
      <SettingRow
        label="Rate Sensitivity"
        tooltip={TOOLTIP.RateSensitivity}
        control={
          <EditableNumber
            value={levels.rate_sensitivity}
            min={0.25}
            max={4}
            step={0.05}
            onCommit={(v) => commitLevels({ rate_sensitivity: v })}
            label="Rate sensitivity"
          />
        }
      >
        <Slider
          min={0.25}
          max={4}
          step={0.05}
          value={[levels.rate_sensitivity]}
          onValueChange={(e) => handleLevels("rate_sensitivity", e[0])}
          onValueCommit={() => handleFeatureAlter(feature)}
        />
      </SettingRow>
      <SettingRow
        label="Rate Wind-down"
        tooltip={TOOLTIP.Rate}
        control={
          <EditableNumber
            value={levels.rate_tune}
            min={0.1}
            max={2}
            step={0.05}
            onCommit={(v) => commitLevels({ rate_tune: v })}
            label="Rate wind-down"
          />
        }
      >
        <Slider
          min={0.1}
          max={2}
          step={0.05}
          value={[levels.rate_tune]}
          onValueChange={(e) => handleLevels("rate_tune", e[0])}
          onValueCommit={() => handleFeatureAlter(feature)}
        />
      </SettingRow>
    </>
  );
}

function Constant() {
  const { feature, handleLevels, handleFeatureAlter, commitLevels } =
    useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <SettingRow
      label="Constant Level"
      tooltip={TOOLTIP.Constant}
      control={
        <EditableNumber
          value={levels.constant_level}
          min={0.01}
          max={1.0}
          step={0.01}
          onCommit={(v) => commitLevels({ constant_level: v })}
          label="Constant level"
        />
      }
    >
      <Slider
        min={0.01}
        max={1.0}
        step={0.01}
        value={[levels.constant_level]}
        onValueChange={(e) => handleLevels("constant_level", e[0])}
        onValueCommit={() => handleFeatureAlter(feature)}
      />
    </SettingRow>
  );
}

// Fills the rest of the feature box and scrolls its contents, so the box
// always ends at the bottom of the window (no hard-coded heights needed)
function FeatureScrollArea({ children }: { children: ReactNode }) {
  return (
    <ScrollArea className="scrollbar min-h-0 flex-1 overflow-y-hidden pr-2 hover:overflow-y-scroll hover:pr-0">
      {children}
    </ScrollArea>
  );
}
