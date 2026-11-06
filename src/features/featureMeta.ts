import type { FeVCFeatureType } from "@bindings/FeVCFeatureType";
import type { FeVCToyFeature } from "@bindings/FeVCToyFeature";
import {
  AudioWaveform,
  Crosshair,
  type LucideIcon,
  Maximize2,
  Minimize2,
  MoveVertical,
  RotateCw,
  Vibrate,
} from "lucide-react";

/** Icon for each kind of toy feature (motor) */
export const featureIcons: Record<FeVCFeatureType, LucideIcon> = {
  Vibrator: Vibrate,
  Rotator: RotateCw,
  Linear: MoveVertical,
  Oscillate: AudioWaveform,
  Constrict: Minimize2,
  Inflate: Maximize2,
  Position: Crosshair,
};

/**
 * Friendly name like "Vibrator 1": numbered from 1 among features of the
 * same type, so a toy with one vibrator and one rotator shows
 * "Vibrator 1" and "Rotator 1".
 */
export function featureLabel(features: FeVCToyFeature[], index: number) {
  const feature = features[index];
  const number = features
    .slice(0, index + 1)
    .filter((f) => f.feature_type === feature.feature_type).length;
  return `${feature.feature_type} ${number}`;
}
