import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import type { FeVCFeatureType } from "../../src-tauri/bindings/FeVCFeatureType";
import { DEBOUNCE_TIME, SIMULATE_TOY_FEATURE } from "../data/constants";
import { useToastContext } from "../context/ToastContext";

export default function useSimulate(
  toyId: number,
  featureIndex: number,
  featureType: FeVCFeatureType
) {
  const [simulate, setSimulate] = useState(false);
  const [simulateLevel, setSimulateLevel] = useState(0.5);
  const toast = useToastContext();

  function simulateHandler() {
    setSimulate((b) => !b);
  }

  function simulateLevelHandler(e: number) {
    setSimulateLevel(e);
  }

  useEffect(() => {
    if (simulate) {
      const t = setTimeout(() => {
        invokeSimulation(simulateLevel);
      }, DEBOUNCE_TIME);
      return () => clearTimeout(t);
    } else {
      invokeSimulation(0);
    }
  }, [simulate, simulateLevel]);

  async function invokeSimulation(floatLevel: number) {
    try {
      await invoke(SIMULATE_TOY_FEATURE, {
        toyId,
        featureIndex,
        featureType,
        floatLevel,
      });
    } catch (e) {
      toast.createToast("Could not simulate device feature!", `${e}`, "error");
    }
  }

  return { simulate, simulateHandler, simulateLevel, simulateLevelHandler };
}
