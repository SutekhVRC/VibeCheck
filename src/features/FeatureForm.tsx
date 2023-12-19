import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { PenetrationSystems, ProcessingModes } from "@/data/stringArrayTypes";
import { Select } from "@/layout/Select";
import { Plus, X } from "lucide-react";
import { ChangeEvent, Fragment, useEffect, useState } from "react";
import { FeProcessingMode } from "src-tauri/bindings/FeProcessingMode";
import { FeToyParameter } from "src-tauri/bindings/FeToyParameter";
import { FeLevelTweaks } from "../../src-tauri/bindings/FeLevelTweaks";
import { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import type { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";
import FourPanel from "../components/FourPanel";
import FourPanelContainer from "../components/FourPanelContainer";
import { OSC_PARAM_PREFIX } from "../data/constants";
import useSimulate from "../hooks/useSimulate";
import { handleFeatureAlter } from "../hooks/useToys";
import Slider from "../layout/Slider";
import Switch from "../layout/Switch";
import { round0 } from "../utils";

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
  const submenuOptions = ["Parameters", "Advanced"] as const;
  type SubmenuOptions = (typeof submenuOptions)[number];
  const [subMenu, setSubMenu] = useState<SubmenuOptions>("Parameters");

  useEffect(() => {
    setToyFeature(toy.features[selectedIndex] ?? toy.features[0]);
  }, [toy, selectedIndex]);

  const {
    simulateEnabled,
    simulateLevel,
    toggleSimulate,
    simulateOnValueChange,
    simulateOnValueCommit,
  } = useSimulate(toy.toy_id, feature);

  function handleBool(checked: boolean, name: keyof FeVCToyFeature) {
    setToyFeature((f) => {
      const newF = { ...f, [name]: checked } as FeVCToyFeature;
      handleFeatureAlter(toy, newF);
      return newF;
    });
  }

  function removeParam(parameter: string) {
    setToyFeature((f) => {
      const newF = {
        ...f,
        osc_parameters: f.osc_parameters.filter(
          (param) => param.parameter != parameter,
        ),
      };
      handleFeatureAlter(toy, newF);
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
      const newParam = `${OSC_PARAM_PREFIX}param-${findParamName(
        f.osc_parameters,
      )}`;
      const newF = {
        ...f,
        osc_parameters: [
          ...f.osc_parameters,
          {
            parameter: newParam,
            processing_mode: "Raw" as FeProcessingMode,
          },
        ],
      };
      handleFeatureAlter(toy, newF);
      return newF;
    });
  }

  function handleOscParam(
    e: ChangeEvent<HTMLInputElement> | ChangeEvent<HTMLSelectElement>,
    paramIndex: number,
  ) {
    setToyFeature((f) => {
      const newParams = [...f.osc_parameters];
      if (e.target.name == "osc_parameter") {
        newParams[paramIndex].parameter =
          `${OSC_PARAM_PREFIX}${e.target.value}`;
      } else if (e.target.name == "osc_parameter_mode") {
        newParams[paramIndex].processing_mode = e.target
          .value as FeProcessingMode;
      }
      const newF = {
        ...f,
        osc_parameters: newParams,
      };
      handleFeatureAlter(toy, newF);
      return newF;
    });
  }

  function handleInputProcessor(e: ChangeEvent<HTMLSelectElement>) {
    setToyFeature((f) => {
      const newF = {
        ...f,
        penetration_system: {
          ...f.penetration_system,
          [e.target.name]: e.target.value,
        },
      };
      handleFeatureAlter(toy, newF);
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

  function handleCommit() {
    handleFeatureAlter(toy, feature);
  }

  return (
    <>
      <FourPanelContainer>
        <FourPanel
          text="Enabled"
          tooltip="Enable/Disable this feature."
          two={
            <Switch
              size="small"
              checked={feature.feature_enabled}
              onCheckedChange={(checked) =>
                handleBool(checked, "feature_enabled")
              }
            />
          }
        />
      </FourPanelContainer>
      <div className="flex gap-4 m-2">
        <Button onClick={() => setSubMenu("Parameters")}>Parameters</Button>
        <Button onClick={() => setSubMenu("Advanced")}>Advanced</Button>
      </div>
      <ScrollArea className="rounded-md border flex flex-grow flex-col h-[calc(100vh-410px)]">
        <>
          {subMenu == "Parameters" ? (
            <div className="flex flex-col justify-between gap-2">
              <div className="h-[calc(100vh-470px)] overflow-y-scroll scrollbar">
                <div className="grid grid-cols-[20fr,_6fr,_1fr] text-sm text-justify p-4 gap-y-2 gap-x-6 items-center">
                  {feature.osc_parameters.map((param, paramIndex) => {
                    // TODO this is an antipatern to use index, however there should only be a low number of params
                    // This needs to be by index because we update directly from backend
                    // If we key on param, it would change on every update when typing, and un-select after each char
                    return (
                      <Fragment key={paramIndex}>
                        <input
                          className="text-zinc-800 px-4 rounded-sm outline-none w-full"
                          name="osc_parameter"
                          value={param.parameter.replace(OSC_PARAM_PREFIX, "")}
                          onChange={(e) => handleOscParam(e, paramIndex)} // Not debounced because :shrug:
                        />
                        <Select
                          name="osc_parameter_mode"
                          value={param.processing_mode}
                          onChange={(e) => {
                            handleOscParam(e, paramIndex);
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
                </div>
              </div>
              <div className="flex flex-col justify-center">
                <div>
                  <Button onClick={() => addParam()} size="sm">
                    <Plus className="h-6" />
                  </Button>
                </div>
              </div>
            </div>
          ) : (
            <FourPanelContainer>
              {feature.feature_type == "Linear" && (
                <FourPanel
                  text="Linear Speed"
                  tooltip="Linear positional duration speed in milliseconds. Speed is determined by the toy itself, this is only requested speed."
                  three={
                    <Slider
                      min={10}
                      max={1000}
                      step={1}
                      value={[levels.linear_position_speed]}
                      onValueChange={(e) =>
                        handleLevels("linear_position_speed", e[0])
                      }
                      onValueCommit={handleCommit}
                    />
                  }
                  four={levels.linear_position_speed.toString()}
                />
              )}
              <FourPanel
                text="Rate Level"
                tooltip="This uses rate mode on the float input."
                three={
                  <Slider
                    min={1}
                    max={20}
                    step={1}
                    value={[levels.rate_tune]}
                    onValueChange={(e) => handleLevels("rate_tune", e[0])}
                    onValueCommit={handleCommit}
                  />
                }
                four={levels.rate_tune.toString()}
              />
              <FourPanel
                text="Smooth Level"
                tooltip="This smooths the float input by queueing the amount set with the slider, then transforming them into one value to send instead. If you aren't sending a lot of floats rapidly over OSC you probably want this disabled completely."
                three={
                  <Slider
                    min={1}
                    max={20}
                    step={1}
                    value={[levels.smooth_rate]}
                    onValueChange={(e) => handleLevels("smooth_rate", e[0])}
                    onValueCommit={handleCommit}
                  />
                }
                four={levels.smooth_rate.toString()}
              />
              <FourPanel
                text="Flip Input"
                tooltip="Some toys use a flipped float input. Enable this if your toy seems to do the opposite motor level you were expecting."
                two={
                  <Switch
                    size="small"
                    checked={feature.flip_input_float}
                    onCheckedChange={(checked) =>
                      handleBool(checked, "flip_input_float")
                    }
                  />
                }
              />
              <FourPanel
                text="Idle"
                tooltip="Set the idle motor speed for this feature. Idle activates when there is no input. Your set idle speed won't activate until you send at least one float value in the valid min/max range you have set."
                flipped={feature.flip_input_float}
                three={
                  <Slider
                    multiply={100}
                    min={0}
                    max={1}
                    step={0.01}
                    value={[levels.idle_level]}
                    onValueChange={(e) => handleLevels("idle_level", e[0])}
                    onValueCommit={handleCommit}
                  />
                }
                four={round0.format(levels.idle_level * 100)}
              />
              <FourPanel
                text="Range"
                tooltip="The minimum/maximum motor speed that will be sent to the feature's motor."
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
                    onValueCommit={handleCommit}
                  />
                }
                four={`${round0.format(
                  levels.minimum_level * 100,
                )}-${round0.format(levels.maximum_level * 100)}`}
              />
              <FourPanel
                text="Processor"
                tooltip="The Input processor for this feature"
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
                      value={
                        feature.penetration_system.pen_system_processing_mode
                      }
                      onChange={(e) => {
                        handleInputProcessor(e);
                      }}
                      options={ProcessingModes}
                    />
                  </div>
                }
              />
              {simulateEnabled != null && (
                <FourPanel
                  text="Simulate"
                  tooltip="Test feature power level."
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
            </FourPanelContainer>
          )}
        </>
      </ScrollArea>
    </>
  );
}
