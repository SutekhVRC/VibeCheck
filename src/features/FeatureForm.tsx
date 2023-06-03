import { ChangeEvent, useState } from "react";
import { OSC_PARAM_PREFIX } from "../data/constants";
import { round0 } from "../utils";
import type { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";
import Slider from "../layout/Slider";
import { TooltipLabel } from "../layout/Tooltip";
import useSimulate from "../hooks/useSimulate";
import Switch from "../layout/Switch";
import { ArrowsRightLeftIcon } from "@heroicons/react/24/solid";
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

  const { simulate, simulateHandler, simulateLevel, simulateLevelHandler } =
    useSimulate(toyId, feature.feature_index, feature.feature_type);

  return (
    <div className="grid grid-cols-[minmax(6rem,_1fr)_1fr_minmax(6rem,_3fr)_1fr] text-sm text-justify gap-y-1 p-4">
      <FourPanel
        one={
          <TooltipLabel text="Enabled" tooltip="Enable/Disable this feature." />
        }
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
        one={
          <TooltipLabel
            text="OSC Parameter"
            tooltip="The float OSC parameter to control this feature's motor."
          />
        }
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
        one={
          <TooltipLabel
            text="Smoothing"
            tooltip="This smooths the float input by queueing the amount set with the slider, then transforming them into one value to send instead. If you aren't sending a lot of floats rapidly over OSC you probably want this disabled completely."
          />
        }
        two={
          <Switch
            size="small"
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
        four={<div className="text-right">{levels.smooth_rate}</div>}
      />
      {feature.feature_type == "Linear" && (
        <FourPanel
          one={
            <TooltipLabel
              text="Linear Speed"
              tooltip="Speed is determined by the toy itself, so this is only requested speed."
            />
          }
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
          four={
            <div className="text-right">{levels.linear_position_speed}</div>
          }
        />
      )}
      <FourPanel
        one={
          <TooltipLabel
            text="Flip Input"
            tooltip="Some toys use a flipped float input. Enable this if your toy seems to do the opposite motor level you were expecting."
          />
        }
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
        one={
          <div className="flex items-center gap-2">
            <TooltipLabel
              text="Idle"
              tooltip="Set the idle motor speed for this feature. Idle activates when there is no input. Your set idle speed won't activate until you send at least one float value in the valid min/max range you have set."
            />
            {feature.flip_input_float && (
              <ArrowsRightLeftIcon className="h-4" />
            )}
          </div>
        }
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
        four={
          <div className="text-right">
            {round0.format(levels.idle_level * 100)}
          </div>
        }
      />
      <FourPanel
        one={
          <div className="flex items-center gap-2">
            <TooltipLabel
              text="Range"
              tooltip="The minimum/maximum motor speed that will be sent to the feature's motor."
            />
            {feature.flip_input_float && (
              <ArrowsRightLeftIcon className="h-4" />
            )}
          </div>
        }
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
        four={
          <div className="text-right">
            {round0.format(levels.minimum_level * 100)}
            {" - "}
            {round0.format(levels.maximum_level * 100)}
          </div>
        }
      />
      <div className="h-2" />
      {simulate != null && (
        <FourPanel
          one={
            <TooltipLabel text="Simulate" tooltip="Test feature power level." />
          }
          two={
            <Switch
              size="small"
              checked={simulate}
              onChange={simulateHandler}
            />
          }
          three={
            <Slider
              disabled={!simulate}
              min={0}
              max={1}
              step={0.01}
              value={[simulateLevel]}
              onValueCommit={(e) => simulateLevelHandler(e[0])}
            />
          }
          four={
            <div className="text-right">
              {round0.format(simulateLevel * 100)}
            </div>
          }
        />
      )}
    </div>
  );
}
