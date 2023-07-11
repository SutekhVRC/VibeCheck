import { ChangeEvent, useEffect, useState } from "react";
import { OSC_PARAM_PREFIX } from "../data/constants";
import { round0 } from "../utils";
import type { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";
import Slider from "../layout/Slider";
import useSimulate from "../hooks/useSimulate";
import Switch from "../layout/Switch";
import { FeLevelTweaks } from "../../src-tauri/bindings/FeLevelTweaks";
import FourPanel from "../components/FourPanel";
import { handleFeatureAlter } from "../hooks/useToys";
import { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import { Select } from "../layout/Select";
import { TooltipLabel } from "../layout/Tooltip";
import FourPanelContainer from "../components/FourPanelContainer";

type ToyFeatureFormProps = {
  toy: FeVCToy;
  selectedIndex: number;
};

export default function FeatureForm({
  toy,
  selectedIndex,
}: ToyFeatureFormProps) {
  const [feature, setToyFeature] = useState(
    toy.features[selectedIndex] ?? toy.features[0]
  );
  const levels = feature.feature_levels;

  const modeOptions = ["None", "Smooth", "Rate"] as const;
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

  function handleMode(option: modeOption) {
    const smooth_enabled = option == "Smooth";
    const rate_enabled = option == "Rate";
    setToyFeature((f) => {
      const newF = {
        ...f,
        smooth_enabled,
        rate_enabled,
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
      <FourPanel
        text="Parameter"
        tooltip="The float OSC parameter to control this feature's motor."
        three={
          <input
            className="text-zinc-800 px-4 rounded-sm outline-none w-full"
            name="osc_parameter"
            value={feature.osc_parameter.replace(OSC_PARAM_PREFIX, "")}
            onChange={handleOscParam} // Not debounced because :shrug:
          />
        }
      />
      {feature.smooth_enabled ? (
        <TooltipLabel
          text="Mode"
          tooltip="This smooths the float input by queueing the amount set with the slider, then transforming them into one value to send instead. If you aren't sending a lot of floats rapidly over OSC you probably want this disabled completely."
        />
      ) : feature.rate_enabled ? (
        <TooltipLabel
          text="Mode"
          tooltip="This uses rate mode on the float input."
        />
      ) : (
        <TooltipLabel
          text="Mode"
          tooltip="No additional mode options will be applied to the float input."
        />
      )}
      <Select
        value={
          feature.rate_enabled
            ? "Rate"
            : feature.smooth_enabled
            ? "Smooth"
            : "None"
        }
        onChange={(e) => {
          handleMode(e.target.value as modeOption);
        }}
        options={modeOptions}
      />
      <div className="col-span-2 md:col-span-1">
        {feature.smooth_enabled ? (
          <Slider
            disabled={!feature.smooth_enabled}
            min={1}
            max={20}
            step={1}
            value={[levels.smooth_rate]}
            onValueChange={(e) => handleLevels("smooth_rate", e[0])}
            onValueCommit={handleCommit}
          />
        ) : feature.rate_enabled ? (
          <Slider
            disabled={!feature.rate_enabled}
            min={0.1}
            max={1}
            step={0.01}
            value={[levels.rate_tune]}
            onValueChange={(e) => handleLevels("rate_tune", e[0])}
            onValueCommit={handleCommit}
          />
        ) : null}
      </div>
      <div className="text-right hidden md:block">
        {feature.smooth_enabled
          ? levels.smooth_rate.toString()
          : feature.rate_enabled
          ? levels.rate_tune.toString()
          : null}
      </div>
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
              onValueChange={(e) => handleLevels("linear_position_speed", e[0])}
              onValueCommit={handleCommit}
            />
          }
          four={levels.linear_position_speed.toString()}
        />
      )}
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
        four={`${round0.format(levels.minimum_level * 100)}-${round0.format(
          levels.maximum_level * 100
        )}`}
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
  );
}
