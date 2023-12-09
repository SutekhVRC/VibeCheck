import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import type { FeCoreEvent } from "../../src-tauri/bindings/FeCoreEvent";
import { FeStateEvent } from "../../src-tauri/bindings/FeStateEvent";
import type { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import { createToast } from "../components/Toast";
import {
  CORE_EVENT,
  DISABLE,
  ENABLE,
  GET_CONFIG,
  SCAN_LENGTH,
  START_SCAN,
  STOP_SCAN,
} from "../data/constants";
import { assertExhaustive } from "../utils";

export function useCoreEvents() {
  const [isEnabled, setIsEnabled] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [config, setConfig] = useState<FeVibeCheckConfig | null>(null);

  async function enable() {
    try {
      await invoke(ENABLE);
      setIsEnabled(true);
    } catch (e) {
      createToast("error", "Could not enable!", JSON.stringify(e));
    }
  }

  async function stopScanAndDisable() {
    try {
      await stopScan();
      await invoke(DISABLE);
      setIsEnabled(false);
    } catch (e) {
      createToast("error", "Could not disable!", JSON.stringify(e));
    }
  }

  async function toggleIsEnabled() {
    if (isEnabled) {
      await stopScanAndDisable();
    } else {
      await enable();
    }
  }

  async function enableAndStartScan() {
    try {
      await enable();
      await invoke(START_SCAN);
      setIsScanning(true);
    } catch (e) {
      createToast("error", "Could not start scan!", JSON.stringify(e));
    }
  }

  async function stopScan() {
    try {
      await invoke(STOP_SCAN);
      setIsScanning(false);
    } catch (e) {
      createToast("error", "Could not stop scan!", JSON.stringify(e));
    }
  }

  async function toggleScan() {
    if (isScanning) {
      await stopScan();
    } else {
      await enableAndStartScan();
    }
  }

  useEffect(() => {
    if (!isScanning) return;
    const i = setInterval(() => stopScan(), SCAN_LENGTH);
    return () => clearInterval(i);
  }, [isScanning]);

  function handleStateEvent(payload: FeStateEvent) {
    switch (payload) {
      case "Disable":
        stopScanAndDisable();
        break;
      case "EnableAndScan":
        enableAndStartScan();
        break;
      default:
        assertExhaustive(payload);
    }
  }

  function handleCoreEvent(payload: FeCoreEvent) {
    switch (payload.kind) {
      case "Scan":
        setIsScanning(payload.data == "Start");
        break;
      case "State":
        handleStateEvent(payload.data);
        break;
      default:
        assertExhaustive(payload);
    }
  }

  useEffect(() => {
    const unlistenPromise = listen<FeCoreEvent>(CORE_EVENT, (event) =>
      handleCoreEvent(event.payload),
    );

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  async function refreshConfig() {
    try {
      const config = await invoke<FeVibeCheckConfig>(GET_CONFIG);
      setConfig(config);
    } catch {
      setConfig(null);
    }
  }

  useEffect(() => {
    async function getConfig() {
      try {
        const config = await invoke<FeVibeCheckConfig>(GET_CONFIG);
        setConfig(config);
      } catch {
        setConfig(null);
      }
    }
    getConfig();
  }, []);

  return {
    isScanning,
    isEnabled,
    toggleIsEnabled,
    toggleScan,
    config,
    refreshConfig,
  };
}
