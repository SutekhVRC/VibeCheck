import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import type { FeVCFeatureType } from "../../src-tauri/bindings/FeVCFeatureType";
import { SIMULATE_TOY_FEATURE } from "../data/constants";
import { createToast } from "../components/Toast";

export default function useSimulate(
  toyId: number | null,
  featureIndex: number,
  featureType: FeVCFeatureType
) {
  const [simulate, setSimulate] = useState(false);
  const [simulateLevel, setSimulateLevel] = useState(0.5);

  function simulateOnChange() {
    setSimulate((b) => {
      const newEnableState = !b;
      if (newEnableState) invokeSimulation(simulateLevel);
      else invokeSimulation(0);
      return !b;
    });
  }

  function simulateOnValueChange(e: number) {
    setSimulateLevel(e);
  }

  function simulateOnValueCommit() {
    if (simulate) invokeSimulation(simulateLevel);
    else invokeSimulation(0);
  }

  useEffect(() => {
    setSimulate(false);
    setSimulateLevel(0.5);
    return () => {
      invokeSimulation(0);
    };
  }, [toyId, featureType, featureIndex]);

  if (toyId == null)
    return {
      simulate: null,
      simulateHandler: () => null,
      simulateLevel: null,
      simulateLevelHandler: () => null,
    };

  async function invokeSimulation(floatLevel: number) {
    if (toyId == null) return;
    try {
      await invoke(SIMULATE_TOY_FEATURE, {
        toyId,
        featureIndex,
        featureType,
        floatLevel,
      });
    } catch (e) {
      createToast(
        "error",
        "Could not simulate device feature!",
        JSON.stringify(e)
      );
    }
  }

  return {
    simulate,
    simulateLevel,
    simulateOnChange,
    simulateOnValueChange,
    simulateOnValueCommit,
  };
}
