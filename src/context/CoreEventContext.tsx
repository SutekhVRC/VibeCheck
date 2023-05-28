import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { createContext, useContext, useEffect, useState } from "react";
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
import type { ReactNode } from "react";
import type { FeCoreEvent } from "../../src-tauri/bindings/FeCoreEvent";
import type { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import { FeStateEvent } from "../../src-tauri/bindings/FeStateEvent";
import { useToastContext } from "./ToastContext";

type CoreContextProps = {
  isScanning: boolean;
  isEnabled: boolean;
  toggleIsEnabled: () => void;
  toggleScan: () => void;
  config: FeVibeCheckConfig | null;
  refreshConfig: () => void;
};
const INITIAL_CORE_STATE: CoreContextProps = {
  isScanning: false,
  isEnabled: false,
  toggleIsEnabled: () => null,
  toggleScan: () => null,
  config: null,
  refreshConfig: () => null,
};
const CoreEventContext = createContext<CoreContextProps>(INITIAL_CORE_STATE);

export function useCoreEventContext() {
  return useContext(CoreEventContext);
}

export function CoreEventProvider({ children }: { children: ReactNode }) {
  const [isEnabled, setIsEnabled] = useState(INITIAL_CORE_STATE.isEnabled);
  const [isScanning, setIsScanning] = useState(INITIAL_CORE_STATE.isScanning);
  const toast = useToastContext();

  const [config, setConfig] = useState<FeVibeCheckConfig | null>(
    INITIAL_CORE_STATE.config
  );

  async function enable() {
    try {
      await invoke(ENABLE);
      setIsEnabled(true);
    } catch (e) {
      toast.createToast("Could not enable!", JSON.stringify(e), "error");
    }
  }

  async function stopScanAndDisable() {
    try {
      stopScan();
      await invoke(DISABLE);
      setIsEnabled(false);
    } catch (e) {
      toast.createToast("Could not disable!", JSON.stringify(e), "error");
    }
  }

  async function toggleIsEnabled() {
    isEnabled ? stopScanAndDisable() : enable();
  }

  async function enableAndStartScan() {
    try {
      enable();
      await invoke(START_SCAN);
      setIsScanning(true);
    } catch (e) {
      toast.createToast("Could not start scan!", JSON.stringify(e), "error");
    }
  }

  async function stopScan() {
    try {
      await invoke(STOP_SCAN);
      setIsScanning(false);
    } catch (e) {
      toast.createToast("Could not stop scan!", JSON.stringify(e), "error");
    }
  }

  function toggleScan() {
    isScanning ? stopScan() : enableAndStartScan();
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
      handleCoreEvent(event.payload)
    );

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // Config here because I don't want the async refresh inside the Settings Dialog
  // Not sure where else to put it
  async function refreshConfig() {
    try {
      const config = await invoke<FeVibeCheckConfig>(GET_CONFIG);
      setConfig(config);
    } catch {
      setConfig(null);
    }
  }
  useEffect(() => {
    refreshConfig();
  }, []);

  return (
    <CoreEventContext.Provider
      value={{
        isScanning,
        isEnabled,
        toggleIsEnabled,
        toggleScan,
        config,
        refreshConfig,
      }}
    >
      {children}
    </CoreEventContext.Provider>
  );
}
