import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";
import { createToast } from "../components/Toast";
import { INVOKE } from "../data/constants";
import { useUpdateEffect } from "./useUpdateEffect";

export default function useSimulate(
  toyId: number | null,
  feature: FeVCToyFeature,
) {
  const [simulateEnabled, setSimulateEnabled] = useState(false);
  const [level, setLevel] = useState(0.5);

  function toggleSimulate() {
    setSimulateEnabled((b) => !b);
  }

  function simulateOnValueChange(e: number) {
    setLevel(e);
  }

  function simulateOnValueCommit() {
    if (feature.feature_levels.idle_level == 0) {
      invokeSimulation(level);
    }
  }

  useEffect(() => {
    setSimulateEnabled(false);
    setLevel(0.5);
    return () => {
      if (feature.feature_levels.idle_level == 0) {
        invokeSimulation(0);
      }
    };
  }, [toyId, feature]);

  useUpdateEffect(() => {
    if (feature.feature_levels.idle_level == 0) {
      if (simulateEnabled) invokeSimulation(level);
      else invokeSimulation(0);
    }
  }, [simulateEnabled, level]);

  async function invokeSimulation(floatLevel: number) {
    if (toyId == null) return;
    try {
      await invoke(INVOKE.SIMULATE_TOY_FEATURE, {
        toyId,
        featureIndex: feature.feature_index,
        featureType: feature.feature_type,
        floatLevel,
        stop: false,
      });
    } catch (e) {
      createToast(
        "error",
        "Could not simulate device feature!",
        JSON.stringify(e),
      );
    }
  }

  if (toyId == null || feature.feature_levels.idle_level > 0)
    return {
      simulateEnabled: null,
      simulateLevel: null,
      toggleSimulate: () => null,
      simultaeOnValueChange: () => null,
      simultaeOnValueCommit: () => null,
    };
  else
    return {
      simulateEnabled,
      simulateLevel: level,
      toggleSimulate,
      simulateOnValueChange,
      simulateOnValueCommit,
    };
}
