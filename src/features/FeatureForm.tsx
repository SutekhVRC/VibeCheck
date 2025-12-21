import { FreeTextOptions } from "@/components/ui/FreeTextOptions";
import { useCoreEventContext } from "@/context/CoreEvents";
import { PenetrationSystems, ProcessingModes } from "@/data/stringArrayTypes";
import { Select } from "@/layout/Select";
import { cn } from "@/lib/utils";
import { FeLevelTweaks } from "@bindings/FeLevelTweaks";
import { FeProcessingMode } from "@bindings/FeProcessingMode";
import { FeToyParameter } from "@bindings/FeToyParameter";
import { FeVCToy } from "@bindings/FeVCToy";
import type { FeVCToyFeature } from "@bindings/FeVCToyFeature";
import { ScrollArea } from "@radix-ui/react-scroll-area";
import { DebouncedFunc, debounce } from "lodash";
import { Plus, X } from "lucide-react";
import {
  ChangeEvent,
  Dispatch,
  Fragment,
  ReactNode,
  SetStateAction,
  createContext,
  useCallback,
  useContext,
  useState,
} from "react";
import FourPanel from "../components/FourPanel";
import FourPanelContainer from "../components/FourPanelContainer";
import { OSC, TOOLTIP } from "../data/constants";
import useSimulate from "../hooks/useSimulate";
import { handleFeatureAlter as handleToyFeatureAlter } from "../hooks/useToys";
import Slider from "../layout/Slider";
import Switch from "../layout/Switch";
import { round0 } from "../utils";

type FeatureFormContextProps = {
  feature: FeVCToyFeature;
  setToyFeature: Dispatch<SetStateAction<FeVCToyFeature>>;
  debouncedAlter: DebouncedFunc<(f: FeVCToyFeature) => void>;
  handleFeatureAlter: (f: FeVCToyFeature) => void;
  handleBool: (checked: boolean, name: keyof FeVCToyFeature) => void;
  handleLevels: (key: keyof FeLevelTweaks, value: number) => void;
};

const FeatureFormContext = createContext<FeatureFormContextProps>({
  feature: {} as FeVCToyFeature,
  setToyFeature: () => null,
  debouncedAlter: debounce(() => null, 1000),
  handleFeatureAlter: () => null,
  handleBool: () => null,
  handleLevels: () => null,
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
};

export default function FeatureForm({
  toy,
  selectedIndex,
}: ToyFeatureFormProps) {
  const [feature, setToyFeature] = useState(
    toy.features[selectedIndex] ?? toy.features[0],
  );
  const levels = feature.feature_levels;
  const { config } = useCoreEventContext();

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
        feature_levels: { ...levels, [key]: value },
      };
    });
  }

  const tweakSliders = feature.osc_parameters.reduce(
    (seenModes, oscParam) => {
      seenModes.add(oscParam.processing_mode);
      return seenModes;
    },
    new Set<string>([feature.penetration_system.pen_system_processing_mode]),
  );

  return (
    <FeatureFormContext.Provider
      value={{
        feature,
        setToyFeature,
        debouncedAlter,
        handleFeatureAlter,
        handleBool,
        handleLevels,
      }}
    >
      <div className="rounded-md bg-zinc-700 p-4">
        <HackyScrollArea>
          <FourPanelContainer>
            <Enabled />
            <InputProcessor />
            <InputFilter />
            <Range />
            {config?.show_feature_advanced && (
              <>
                <Idle />
                <FlipInput />
                {feature.feature_type == "Linear" && <Linear />}
                {tweakSliders.has("Smooth") && <Smooth />}
                {tweakSliders.has("Rate") && <Rate />}
                {tweakSliders.has("Constant") && <Constant />}
                <Simulate toy={toy} />
              </>
            )}
          </FourPanelContainer>
          <div className="text-left">Parameters</div>
          <FourPanelContainer>
            <Parameters />
          </FourPanelContainer>
        </HackyScrollArea>
      </div>
    </FeatureFormContext.Provider>
  );
}

function Enabled() {
  const { feature, handleBool } = useFeatureFormContext();

  return (
    <FourPanel
      text="Enabled"
      tooltip={TOOLTIP.Enabled}
      two={
        <Switch
          size="small"
          checked={feature.feature_enabled}
          onCheckedChange={(checked) => handleBool(checked, "feature_enabled")}
        />
      }
    />
  );
}

function InputProcessor() {
  const { feature, setToyFeature, handleFeatureAlter } =
    useFeatureFormContext();

  function handleInputProcessor(e: ChangeEvent<HTMLSelectElement>) {
    setToyFeature((f) => {
      const newF = {
        ...f,
        penetration_system: {
          ...f.penetration_system,
          [e.target.name]: e.target.value,
        },
      };
      handleFeatureAlter(newF);
      return newF;
    });
  }

  return (
    <FourPanel
      text="Processor"
      tooltip={TOOLTIP.InputProcessor}
      three={
        <div className="flex gap-2">
          <Select
            name="pen_system_type"
            value={feature.penetration_system.pen_system_type}
            onChange={(e) => {
              handleInputProcessor(e);
            }}
            options={PenetrationSystems}
          />
          <Select
            name="pen_system_processing_mode"
            value={feature.penetration_system.pen_system_processing_mode}
            onChange={(e) => {
              handleInputProcessor(e);
            }}
            options={ProcessingModes}
          />
        </div>
      }
    />
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
      newParams[paramIndex].parameter = normalizeOscParameter(e.target.value);
      const newF = {
        ...f,
        osc_parameters: newParams,
      };
      debouncedAlter(newF);
      return newF;
    });
  }

  function handleOscParamMode(
    e: ChangeEvent<HTMLSelectElement>,
    paramIndex: number,
  ) {
    setToyFeature((f) => {
      const newParams = [...f.osc_parameters];
      newParams[paramIndex].processing_mode = e.target
        .value as FeProcessingMode;
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
    <>
      {/* <div className="grid grid-cols-[minmax(6rem,20fr),minmax(6rem,6fr),minmax(1rem,1fr)] gap-x-6 gap-y-2 text-justify text-sm"> */}
      {feature.osc_parameters.map((param, paramIndex) => {
        // TODO: Using index is generally an anti-pattern, but I think it's required in this specific scenario
        // If we key on a parameter or other identifiers, typing the parameter name would trigger a refresh from the backend
        // This would then deselect the input element while typing
        return (
          <Fragment key={paramIndex}>
            {/* Adding debounce on this makes it more complex b/c separate state, plus parent key on index */}
            <input
              className="col-span-1 w-full rounded-sm px-4 text-zinc-800 outline-none md:col-span-2"
              name="osc_parameter"
              value={param.parameter.replace(OSC.PARAM_PREFIX, "")}
              onChange={(e) => handleOscParam(e, paramIndex)}
            />
            <Select
              name="osc_parameter_mode"
              value={param.processing_mode}
              onChange={(e) => {
                handleOscParamMode(e, paramIndex);
              }}
              options={ProcessingModes}
            />
            <button
              className="flex justify-center"
              onClick={() => removeParam(param.parameter)}
            >
              <X className="h-5" />
            </button>
          </Fragment>
        );
      })}
      <div className="col-span-3"></div>
      <div className="col-span-2 flex justify-center md:col-span-1">
        <button onClick={addParam}>
          <Plus className="h-5" />
        </button>
      </div>
    </>
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
    <FourPanel
      text="Input Filter"
      tooltip={TOOLTIP.InputFilter}
      three={
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
      }
    />
  );
}

function Linear() {
  const { feature, handleLevels, handleFeatureAlter } = useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <FourPanel
      text="Linear Speed"
      tooltip={TOOLTIP.LinearSpeed}
      three={
        <Slider
          min={10}
          max={1000}
          step={1}
          value={[levels.linear_position_speed]}
          onValueChange={(e) => handleLevels("linear_position_speed", e[0])}
          onValueCommit={() => handleFeatureAlter(feature)}
        />
      }
      four={levels.linear_position_speed.toString()}
    />
  );
}

function Idle() {
  const { feature, handleLevels, handleFeatureAlter } = useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <FourPanel
      text="Idle Speed"
      tooltip={TOOLTIP.Idle}
      flipped={feature.flip_input_float}
      three={
        <Slider
          multiply={100}
          min={0}
          max={1}
          step={0.01}
          value={[levels.idle_level]}
          onValueChange={(e) => handleLevels("idle_level", e[0])}
          onValueCommit={() => handleFeatureAlter(feature)}
        />
      }
      four={round0.format(levels.idle_level * 100)}
    />
  );
}

function Range() {
  const { feature, setToyFeature, handleFeatureAlter } =
    useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <FourPanel
      text="Range"
      tooltip={TOOLTIP.Range}
      flipped={feature.flip_input_float}
      three={
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
                  ...levels,
                  minimum_level: e[0],
                  maximum_level: e[1],
                },
              };
            });
          }}
          onValueCommit={() => handleFeatureAlter(feature)}
        />
      }
      four={`${round0.format(levels.minimum_level * 100)}-${round0.format(
        levels.maximum_level * 100,
      )}`}
    />
  );
}

function FlipInput() {
  const { feature, handleBool } = useFeatureFormContext();
  return (
    <FourPanel
      text="Flip Input"
      tooltip={TOOLTIP.FlipInput}
      two={
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
  const { feature, handleLevels, handleFeatureAlter } = useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <FourPanel
      text="Smooth Level"
      tooltip={TOOLTIP.Smooth}
      three={
        <Slider
          min={1}
          max={20}
          step={1}
          value={[levels.smooth_rate]}
          onValueChange={(e) => handleLevels("smooth_rate", e[0])}
          onValueCommit={() => handleFeatureAlter(feature)}
        />
      }
      four={levels.smooth_rate.toString()}
    />
  );
}

function Rate() {
  const { feature, handleLevels, handleFeatureAlter } = useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <FourPanel
      text="Rate Level"
      tooltip={TOOLTIP.Rate}
      three={
        <Slider
          min={0.1}
          max={2}
          step={0.05}
          value={[levels.rate_tune]}
          onValueChange={(e) => handleLevels("rate_tune", e[0])}
          onValueCommit={() => handleFeatureAlter(feature)}
        />
      }
      four={levels.rate_tune.toString()}
    />
  );
}

function Constant() {
  const { feature, handleLevels, handleFeatureAlter } = useFeatureFormContext();
  const levels = feature.feature_levels;
  return (
    <FourPanel
      text="Constant Level"
      tooltip={TOOLTIP.Constant}
      three={
        <Slider
          min={0.01}
          max={1.0}
          step={0.01}
          value={[levels.constant_level]}
          onValueChange={(e) => handleLevels("constant_level", e[0])}
          onValueCommit={() => handleFeatureAlter(feature)}
        />
      }
      four={levels.constant_level.toString()}
    />
  );
}

function Simulate({ toy }: { toy: FeVCToy }) {
  const { feature } = useFeatureFormContext();
  const {
    simulateEnabled,
    simulateLevel,
    toggleSimulate,
    simulateOnValueChange,
    simulateOnValueCommit,
  } = useSimulate(toy.toy_id, feature);
  return (
    <>
      {simulateEnabled !== null && (
        <FourPanel
          text="Simulate"
          tooltip={TOOLTIP.Simulate}
          flipped={feature.flip_input_float}
          two={
            <Switch
              size="small"
              checked={simulateEnabled}
              onCheckedChange={toggleSimulate}
            />
          }
          three={
            <Slider
              multiply={100}
              disabled={!simulateEnabled}
              min={0}
              max={1}
              step={0.01}
              value={[simulateLevel]}
              onValueChange={(e) => simulateOnValueChange(e[0])}
              onValueCommit={() => simulateOnValueCommit()}
            />
          }
          four={round0.format(simulateLevel * 100)}
        />
      )}
    </>
  );
}

function HackyScrollArea({ children }: { children: ReactNode }) {
  // extremely hacky
  const { config } = useCoreEventContext();
  return (
    <ScrollArea
      className={cn(
        "scrollbar overflow-y-hidden pr-2 hover:overflow-y-scroll hover:pr-0",
        config?.show_toy_advanced
          ? "h-[calc(100vh-300px)] md:h-[calc(100vh-280px)]"
          : "h-[calc(100vh-200px)]",
      )}
    >
      {children}
    </ScrollArea>
  );
}
