import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Select } from "@/layout/Select";
import { PlusIcon, XMarkIcon } from "@heroicons/react/24/solid";
import { ChangeEvent, useEffect, useState } from "react";
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

  const modeOptions = ["Raw", "Smooth", "Rate", "Constant"] as const;
  type modeOption = (typeof modeOptions)[number];

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

  function handleMode(parameter: string, option: modeOption) {
    setToyFeature((f) => {
      const newF = {
        ...f,
        osc_parameters: {
          ...f.osc_parameters,
          [parameter]: {
            parameter: parameter,
            processing_mode: option,
          },
        },
      };
      handleFeatureAlter(toy, newF);
      return newF;
    });
  }

  function removeParam(parameter: string) {
    setToyFeature((f) => {
      const {
        osc_parameters: { [parameter]: _, ...restNewParams },
        ...restOuterFeature
      } = f;
      const newF = { ...restOuterFeature, osc_parameters: restNewParams };
      handleFeatureAlter(toy, newF);
      return newF;
    });
  }

  function addParam() {
    setToyFeature((f) => {
      const newParam = `${OSC_PARAM_PREFIX}newParam`;
      const newF = {
        ...f,
        osc_parameters: {
          ...f.osc_parameters,
          [newParam]: {
            parameter: newParam,
            processing_mode: "Raw",
          },
        },
      } as FeVCToyFeature;
      handleFeatureAlter(toy, newF);
      return newF;
    });
  }

  function handleOscParam(e: ChangeEvent<HTMLInputElement>) {
    setToyFeature((f) => {
      const newF = {
        ...f,
        [e.target.name]: `${OSC_PARAM_PREFIX}${e.target.value}`,
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
      <ScrollArea className="rounded-md border flex flex-grow flex-col">
        <>
          {subMenu == "Parameters" ? (
            <div className="flex flex-grow flex-col">
              <FourPanelContainer>
                {Object.values(feature.osc_parameters).map((param) => {
                  return (
                    <FourPanel
                      key={param.parameter}
                      text="Parameter"
                      tooltip="The float OSC parameter to control this feature's motor."
                      two={
                        <Select
                          value={param.processing_mode}
                          onChange={(e) => {
                            handleMode(
                              param.parameter,
                              e.target.value as modeOption,
                            );
                          }}
                          options={modeOptions}
                        />
                      }
                      three={
                        <input
                          className="text-zinc-800 px-4 rounded-sm outline-none w-full"
                          name="osc_parameter"
                          value={param.parameter.replace(OSC_PARAM_PREFIX, "")}
                          onChange={handleOscParam} // Not debounced because :shrug:
                        />
                      }
                      four={
                        <button
                          className="flex items-center"
                          onClick={() => removeParam(param.parameter)}
                        >
                          <XMarkIcon className="h-5" />
                        </button>
                      }
                    />
                  );
                })}
              </FourPanelContainer>
              <Button onClick={() => addParam()}>
                <PlusIcon className="h-6" />
              </Button>
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
