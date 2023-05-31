import { ChangeEvent, useEffect, useState } from "react";
import { DEBOUNCE_TIME, OSC_PARAM_PREFIX } from "../data/constants";
import { round0 } from "../utils";
import type { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";
import Slider from "../layout/Slider";
import { TooltipLabel } from "../layout/Tooltip";
import useSimulate from "../hooks/useSimulate";
import Switch from "../layout/Switch";
import { FeLevelTweaks } from "../../src-tauri/bindings/FeLevelTweaks";
import { debounce } from "lodash";

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

  useEffect(() => {
    if (JSON.stringify(feature) === JSON.stringify(oldFeature)) return;
    const debouncedAlter = debounce(
      () => handleFeatureAlter(feature),
      DEBOUNCE_TIME
    );
    if (
      oldFeature.feature_enabled != feature.feature_enabled ||
      oldFeature.flip_input_float != feature.flip_input_float ||
      oldFeature.smooth_enabled != feature.smooth_enabled
    )
      handleFeatureAlter(feature);
    else debouncedAlter();
    return () => debouncedAlter.cancel();
  }, [feature]);

  const handleBool = (checked: boolean, name: keyof FeVCToyFeature) => {
    setToyFeature((feature) => {
      return {
        ...feature,
        [name]: checked,
      };
    });
  };

  function handleOscParam(e: ChangeEvent<HTMLInputElement>) {
    setToyFeature((feature) => {
      return {
        ...feature,
        [e.target.name]: `${OSC_PARAM_PREFIX}${e.target.value}`,
      };
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

  const { simulate, simulateHandler, simulateLevel, simulateLevelHandler } =
    useSimulate(toyId, feature.feature_index, feature.feature_type);

  return (
    <div className="grid grid-cols-[minmax(6rem,_1fr)_1fr_minmax(6rem,_3fr)_1fr] text-sm text-justify gap-y-1 p-4">
      <TooltipLabel text="Enabled" tooltip="Enable/Disable this feature." />
      <Switch
        size="small"
        isEnabled={feature.feature_enabled}
        toggleIsEnabled={(checked: boolean) =>
          handleBool(checked, "feature_enabled")
        }
      />
      <div></div>
      <div></div>
      <TooltipLabel
        text="OSC Parameter"
        tooltip="The float OSC parameter to control this feature's motor."
      />
      <div></div>
      <input
        className="text-zinc-800 px-4 rounded-sm outline-none"
        name="osc_parameter"
        value={feature.osc_parameter.replace(OSC_PARAM_PREFIX, "")}
        onChange={handleOscParam}
      />
      <div></div>
      <TooltipLabel
        text="Smoothing"
        tooltip="This smooths the float input by queueing the amount set with the slider, then transforming them into one value to send instead. If you aren't sending a lot of floats rapidly over OSC you probably want this disabled completely."
      />
      <Switch
        size="small"
        isEnabled={feature.smooth_enabled}
        toggleIsEnabled={(checked: boolean) =>
          handleBool(checked, "smooth_enabled")
        }
      />
      <Slider
        disabled={!feature.smooth_enabled}
        min={1}
        max={20}
        step={1}
        value={[levels.smooth_rate]}
        onValueChange={(e) => handleLevels("smooth_rate", e[0])}
      />
      <div className="text-right">{levels.smooth_rate}</div>
      <TooltipLabel
        text="Linear Speed"
        tooltip="Speed is determined by the toy itself, so this is only requested speed."
      />
      <div></div>
      <Slider
        min={10}
        max={1000}
        step={1}
        value={[levels.linear_position_speed]}
        onValueChange={(e) => handleLevels("linear_position_speed", e[0])}
      />
      <div className="text-right">{levels.linear_position_speed}</div>
      <TooltipLabel
        text="Flip Input"
        tooltip="Some toys use a flipped float input. Enable this if your toy seems to do the opposite motor level you were expecting."
      />
      <Switch
        size="small"
        isEnabled={feature.flip_input_float}
        toggleIsEnabled={(checked: boolean) =>
          handleBool(checked, "flip_input_float")
        }
      />
      <div></div>
      <div></div>
      <TooltipLabel
        text="Idle"
        tooltip="Set the idle motor speed for this feature. Idle activates when there is no input. Your set idle speed won't activate until you send at least one float value in the valid min/max range you have set."
      />
      <div></div>
      <Slider
        dir={feature.flip_input_float ? "rtl" : "ltr"}
        min={0}
        max={1}
        step={0.01}
        value={[levels.idle_level]}
        onValueChange={(e) => handleLevels("idle_level", e[0])}
      />
      <div className="text-right">{round0.format(levels.idle_level * 100)}</div>
      <TooltipLabel
        text="Range"
        tooltip="The minimum/maximum motor speed that will be sent to the feature's motor."
      />
      <div></div>
      <Slider
        dir={feature.flip_input_float ? "rtl" : "ltr"}
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
      />
      <div className="text-right">
        {round0.format(levels.minimum_level * 100)}
        {" - "}
        {round0.format(levels.maximum_level * 100)}
      </div>
      {simulate != null && (
        <>
          <div className="h-2" />
          <div />
          <div />
          <div />
          <TooltipLabel text="Simulate" tooltip="Test feature power level." />
          <Switch
            size="small"
            isEnabled={simulate}
            toggleIsEnabled={simulateHandler}
          />
          <Slider
            disabled={!simulate}
            min={0}
            max={1}
            step={0.01}
            value={[simulateLevel]}
            onValueChange={(e) => simulateLevelHandler(e[0])}
          />
          <div className="text-right">{round0.format(simulateLevel * 100)}</div>
        </>
      )}
    </div>
  );
}
