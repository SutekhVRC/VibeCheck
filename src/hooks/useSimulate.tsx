import { invoke } from "@tauri-apps/api";
import { useEffect, useRef, useState } from "react";
import { SIMULATE_TOY_FEATURE } from "../data/constants";
import { createToast } from "../components/Toast";
import { type FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";

export default function useSimulate(
  toyId: number | null,
  feature: FeVCToyFeature
) {
  const [simulateEnabled, setSimulateEnabled] = useState(false);
  const [simulateLevel, setSimulateLevel] = useState(0.5);
  const hasEnabledRef = useRef(false);
  const featureKey = `${feature.feature_type} ${feature.feature_index}`;

  function toggleSimulate() {
    setSimulateEnabled((b) => !b);
    hasEnabledRef.current = true;
    invokeSimulation(!simulateEnabled ? simulateLevel : 0);
  }

  function simulateOnValueChange(e: number) {
    setSimulateLevel(e);
  }

  function simulateCommit() {
    invokeSimulation(simulateEnabled ? simulateLevel : 0);
  }

  useEffect(() => {
    if (hasEnabledRef.current && feature.feature_levels.idle_level > 0) {
      invokeSimulation(0, true);
      hasEnabledRef.current = false;
    }
  }, [feature.feature_levels.idle_level]);

  useEffect(() => {
    setSimulateEnabled(false);
    setSimulateLevel(0.5);
    return () => {
      if (hasEnabledRef.current) {
        invokeSimulation(0);
      }
      hasEnabledRef.current = false;
    };
  }, [toyId, featureKey]);

  async function invokeSimulation(floatLevel: number, stop = false) {
    if (toyId == null) return;
    try {
      await invoke(SIMULATE_TOY_FEATURE, {
        toyId,
        featureIndex: feature.feature_index,
        featureType: feature.feature_type,
        floatLevel,
        stop,
      });
    } catch (e) {
      createToast(
        "error",
        "Could not simulate device feature!",
        JSON.stringify(e)
      );
    }
  }

  if (toyId == null || feature.feature_levels.idle_level > 0)
    return {
      simulateEnabled: null,
      simulateLevel: null,
      toggleSimulate: () => null,
      simulateOnValueChange: () => null,
      simulateCommit: () => null,
    };
  else
    return {
      simulateEnabled,
      simulateLevel,
      toggleSimulate,
      simulateOnValueChange,
      simulateCommit,
    };
}
