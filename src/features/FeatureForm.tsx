import { ChangeEvent, useState } from "react";
import { OSC_PARAM_PREFIX } from "../data/constants";
import { round0 } from "../utils";
import type { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";
import Slider from "../layout/Slider";
import useSimulate from "../hooks/useSimulate";
import Switch from "../layout/Switch";
import { FeLevelTweaks } from "../../src-tauri/bindings/FeLevelTweaks";
import FourPanel from "../components/FourPanel";

type ToyFeatureFormProps = {
  handleFeatureAlter: (newFeature: FeVCToyFeature) => void;
  toyId: number | null;
  oldFeature: FeVCToyFeature;
};

export default function FeatureForm({
  handleFeatureAlter,
  toyId,
  oldFeature,
}: ToyFeatureFormProps) {
  const [feature, setToyFeature] = useState(oldFeature);
  const levels = feature.feature_levels;

  const handleBool = (checked: boolean, name: keyof FeVCToyFeature) => {
    setToyFeature((f) => {
      const newF = { ...f, [name]: checked };
      handleFeatureAlter(newF);
      return newF;
    });
  };

  function handleOscParam(e: ChangeEvent<HTMLInputElement>) {
    setToyFeature((f) => {
      const newF = {
        ...f,
        [e.target.name]: `${OSC_PARAM_PREFIX}${e.target.value}`,
      };
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

  function handleCommit() {
    handleFeatureAlter(feature);
  }

  const {
    simulate,
    simulateLevel,
    simulateOnChange,
    simulateOnValueChange,
    simulateOnValueCommit,
  } = useSimulate(toyId, feature.feature_index, feature.feature_type);

  return (
    <div className="grid grid-cols-[minmax(6rem,_1fr)_1fr_minmax(6rem,_3fr)_1fr] text-sm text-justify gap-y-1 p-4">
      <FourPanel
        text="Enabled"
        tooltip="Enable/Disable this feature."
        two={
          <Switch
            size="small"
            checked={feature.feature_enabled}
            onChange={(checked: boolean) =>
              handleBool(checked, "feature_enabled")
            }
          />
        }
      />
      <FourPanel
        text="OSC Parameter"
        tooltip="The float OSC parameter to control this feature's motor."
        three={
          <input
            className="text-zinc-800 px-4 rounded-sm outline-none"
            name="osc_parameter"
            value={feature.osc_parameter.replace(OSC_PARAM_PREFIX, "")}
            onChange={handleOscParam} // Not debounced because :shrug:
          />
        }
      />
      <FourPanel
        text="Smoothing"
        tooltip="This smooths the float input by queueing the amount set with the slider, then transforming them into one value to send instead. If you aren't sending a lot of floats rapidly over OSC you probably want this disabled completely."
        two={
          <Switch
            size="small"
            disabled={feature.rate_enabled}
            checked={feature.smooth_enabled}
            onChange={(checked: boolean) =>
              handleBool(checked, "smooth_enabled")
            }
          />
        }
        three={
          <Slider
            disabled={!feature.smooth_enabled}
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
        text="Rate Mode"
        tooltip="Cannot use rate mode and smoothing at the same time."
        two={
          <Switch
            size="small"
            disabled={feature.smooth_enabled}
            checked={feature.rate_enabled}
            onChange={(checked: boolean) => handleBool(checked, "rate_enabled")}
          />
        }
      />
      {feature.feature_type == "Linear" && (
        <FourPanel
          text="Linear Speed"
          tooltip="Speed is determined by the toy itself, so this is only requested speed."
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
            onChange={(checked: boolean) =>
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
        four={`${round0.format(levels.minimum_level * 100)} - ${round0.format(
          levels.maximum_level * 100
        )}`}
      />
      {simulate != null && (
        <FourPanel
          text="Simulate"
          tooltip="Test feature power level."
          flipped={feature.flip_input_float}
          two={
            <Switch
              size="small"
              checked={simulate}
              onChange={simulateOnChange}
            />
          }
          three={
            <Slider
              disabled={!simulate}
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
    </div>
  );
}
