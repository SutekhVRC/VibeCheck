import { invoke } from "@tauri-apps/api";
import type { ChangeEvent } from "react";
import { useMemo } from "react";
import { useEffect, useState } from "react";
import {
  ALTER_TOY,
  DEBOUNCE_TIME,
  OSC_PARAM_PREFIX,
} from "../../data/constants";
import { round0 } from "../../utils";
import type { FeVCToyFeature } from "../../../src-tauri/bindings/FeVCToyFeature";
import Slider from "../../layout/Slider";
import TooltipLabel from "../../layout/Tooltip/TooltipLabel";
import useSimulate from "../../hooks/useSimulate";
import Switch from "../../layout/Switch";
import { useToastContext } from "../../context/ToastContext";

type ToyFeatureFormProps = {
  toyId: number;
  toyFeature: FeVCToyFeature;
};

export default function FeatureForm({
  toyId,
  toyFeature,
}: ToyFeatureFormProps) {
  const { feature_levels: initLevels, ...initFeature } = toyFeature;
  const [feature, setFeature] = useState(initFeature);
  const [levels, setLevels] = useState(initLevels);
  const newFeature = useMemo(() => {
    return { ...feature, feature_levels: levels };
  }, [feature, levels]);
  const toast = useToastContext()

  useEffect(() => {
    if (feature == initFeature) return;
    if (feature.osc_parameter == initFeature.osc_parameter) {
      alterToy(toyId, newFeature);
    } else {
      // Debounce text input
      const t = setTimeout(() => {
        alterToy(toyId, newFeature);
      }, DEBOUNCE_TIME);
      return () => clearTimeout(t);
    }
  }, [feature]);

  useEffect(() => {
    if (levels == initLevels) return;
    // Debounce all level changes
    const t = setTimeout(() => {
      alterToy(toyId, newFeature);
    }, DEBOUNCE_TIME);
    return () => clearTimeout(t);
  }, [levels]);

  async function alterToy(toyId: number, newFeature: FeVCToyFeature) {
    try {
      await invoke(ALTER_TOY, {
        toyId: toyId,
        mutate: { Feature: newFeature },
      });
    } catch (e) {
      toast.createToast(
        "Alter Toy",
        `Could not alter toy!\n${e}`,
        "error"
      );
    }
  }

  const onCheckSwitch = (checked: boolean, name: keyof FeVCToyFeature) => {
    setFeature({ ...feature, [name]: checked });
  };

  function handleOscParam(e: ChangeEvent<HTMLInputElement>) {
    setFeature({
      ...feature,
      [e.target.name]: `${OSC_PARAM_PREFIX}${e.target.value}`,
    });
  }

  function handleLevels(key: string, value: number) {
    setLevels({ ...levels, [key]: value });
  }

  const { simulate, simulateHandler, simulateLevel, simulateLevelHandler } =
    useSimulate(toyId, toyFeature.feature_index, toyFeature.feature_type);

  return (
    <div className="grid grid-cols-[minmax(6rem,_1fr)_1fr_minmax(6rem,_3fr)_1fr] text-sm text-justify gap-y-1 p-4">
      <TooltipLabel text="Enabled" tooltip="Enable/Disable this feature." />
      <Switch
        size="small"
        isEnabled={feature.feature_enabled}
        toggleIsEnabled={(checked: boolean) =>
          onCheckSwitch(checked, "feature_enabled")
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
        className="text-zinc-800 text-xs"
        name="osc_parameter"
        value={feature.osc_parameter.replace(OSC_PARAM_PREFIX, "")}
        onChange={handleOscParam}
      />
      <div></div>
      <TooltipLabel
        text="Flip Input"
        tooltip="Some toys use a flipped float input. Enable this if your toy seems to do the opposite motor level you were expecting."
      />
      <Switch
        size="small"
        isEnabled={feature.flip_input_float}
        toggleIsEnabled={(checked: boolean) =>
          onCheckSwitch(checked, "flip_input_float")
        }
      />
      <div></div>
      <div></div>
      <TooltipLabel
        text="Smoothing"
        tooltip="This smooths the float input by queueing the amount set with the slider, then transforming them into one value to send instead. If you aren't sending a lot of floats rapidly over OSC you probably want this disabled completely."
      />
      <Switch
        size="small"
        isEnabled={feature.smooth_enabled}
        toggleIsEnabled={(checked: boolean) =>
          onCheckSwitch(checked, "smooth_enabled")
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
        text="Idle"
        tooltip="Set the idle motor speed for this feature. Idle activates when there is no input. Your set idle speed won't activate until you send at least one float value in the valid min/max range you have set."
      />
      <div></div>
      <Slider
        min={0}
        max={1}
        step={0.01}
        value={[levels.idle_level]}
        onValueChange={(e) => handleLevels("idle_level", e[0])}
      />
      <div className="text-right">{round0.format(levels.idle_level * 100)}</div>
      <TooltipLabel
        text="Minimum"
        tooltip="The minimum motor speed that will be sent to the feature's motor."
      />
      <div></div>
      <Slider
        min={0}
        max={1}
        step={0.01}
        value={[levels.minimum_level]}
        onValueChange={(e) => handleLevels("minimum_level", e[0])}
      />
      <div className="text-right">
        {round0.format(levels.minimum_level * 100)}
      </div>
      <TooltipLabel
        text="Maximum"
        tooltip="The maximum motor speed that will be sent to the feature's motor."
      />
      <div></div>
      <Slider
        min={0}
        max={1}
        step={0.01}
        value={[levels.maximum_level]}
        onValueChange={(e) => handleLevels("maximum_level", e[0])}
      />
      <div className="text-right">
        {round0.format(levels.maximum_level * 100)}
      </div>
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
        min={0}
        max={1}
        step={0.01}
        value={[simulateLevel]}
        onValueChange={(e) => simulateLevelHandler(e[0])}
      />
      <div className="text-right">{round0.format(simulateLevel * 100)}</div>
    </div>
  );
}
