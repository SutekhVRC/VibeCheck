import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import { ALTER_TOY, ALTER_TOY_DEBOUNCE } from "../../data/constants";
import { round0 } from "../../utils";
import type { FeVCToyFeature } from "../../../src-tauri/bindings/FeVCToyFeature";
import Slider from "../Slider";

type ToyFeatureFormProps = {
  toyId: number;
  feature: FeVCToyFeature;
};

async function setFeature(toyId: number, newFeature: FeVCToyFeature) {
  await invoke(ALTER_TOY, {
    toyId: toyId,
    mutate: { Feature: newFeature },
  });
}

export default function ({ toyId, feature }: ToyFeatureFormProps) {
  // Would like to make this one state, but radix-ui slider uses onValueChange(e: number[])
  // Not sure how to dispatch handlers w/o name, and debounce inside handler?
  const [enabled, setEnabled] = useState(feature.feature_enabled);
  const [oscParameter, setOscParameter] = useState(feature.osc_parameter);
  const [flipInput, setFlipInput] = useState(feature.flip_input_float);
  const [smoothing, setSmoothing] = useState(feature.smooth_enabled);
  const levels = feature.feature_levels;
  const [smoothValue, setSmoothValue] = useState(levels.smooth_rate);
  const [idle, setIdle] = useState(levels.idle_level);
  const [min, setMin] = useState(levels.minimum_level);
  const [max, setMax] = useState(levels.maximum_level);

  const newFeature: FeVCToyFeature = {
    ...feature,
    feature_enabled: enabled,
    osc_parameter: oscParameter,
    smooth_enabled: smoothing,
    flip_input_float: flipInput,
    feature_levels: {
      ...levels,
      smooth_rate: smoothValue,
      idle_level: idle,
      minimum_level: min,
      maximum_level: max,
    },
  };

  useEffect(() => {
    if (JSON.stringify(feature) == JSON.stringify(newFeature)) return;
    setFeature(toyId, newFeature);
  }, [enabled, smoothing, flipInput]);

  useEffect(() => {
    if (JSON.stringify(feature) == JSON.stringify(newFeature)) return;
    const t = setTimeout(() => {
      setFeature(toyId, newFeature);
    }, ALTER_TOY_DEBOUNCE);
    return () => clearTimeout(t);
  }, [oscParameter, smoothValue, idle, min, max]);

  return (
    <div className="grid grid-cols-[minmax(6rem,_1fr)_1fr_minmax(3rem,_10fr)_1fr] text-sm text-justify gap-x-2 gap-y-1 p-4">
      <label>Enabled</label>
      <input
        type="checkbox"
        checked={enabled}
        onChange={() => setEnabled((e) => !e)}
      />
      <div></div>
      <div></div>
      <label>OSC Parameter</label>
      <div></div>
      <input
        className="text-zinc-800 text-xs"
        value={oscParameter}
        onChange={(e) => setOscParameter(e.target.value)}
      />
      <div></div>
      <label>Flip Input</label>
      <input
        type="checkbox"
        checked={flipInput}
        onChange={() => setFlipInput((e) => !e)}
      />
      <div></div>
      <div></div>
      <label>Smoothing</label>
      <input
        type="checkbox"
        checked={smoothing}
        onChange={() => setSmoothing((e) => !e)}
      />
      <Slider
        disabled={!smoothing}
        min={1}
        max={20}
        step={1}
        value={[smoothValue]}
        onValueChange={(e) => setSmoothValue(e[0])}
      />
      <div className="text-right">{smoothValue}</div>
      <label>Idle</label>
      <div></div>
      <Slider
        min={0}
        max={1}
        step={0.01}
        value={[idle]}
        onValueChange={(e) => setIdle(e[0])}
      />
      <div className="text-right">{round0.format(idle * 100)}</div>
      <label>Minimum</label>
      <div></div>
      <Slider
        min={0}
        max={1}
        step={0.01}
        value={[min]}
        onValueChange={(e) => setMin(e[0])}
      />
      <div className="text-right">{round0.format(min * 100)}</div>
      <label>Maximum</label>
      <div></div>
      <Slider
        min={0}
        max={1}
        step={0.01}
        value={[max]}
        onValueChange={(e) => setMax(e[0])}
      />
      <div className="text-right">{round0.format(max * 100)}</div>
    </div>
  );
}
