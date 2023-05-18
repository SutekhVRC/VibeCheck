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

  const [config, setConfig] = useState<FeVibeCheckConfig | null>(
    INITIAL_CORE_STATE.config
  );

  async function enable() {
    await invoke(ENABLE)
      .then(() => setIsEnabled(true))
      .catch((e: string) => alert(e));
  }

  async function stopScanAndDisable() {
    stopScan();
    await invoke(DISABLE);
    setIsEnabled(false);
  }

  async function toggleIsEnabled() {
    isEnabled ? stopScanAndDisable() : enable();
  }

  async function enableAndStartScan() {
    enable();
    await invoke(START_SCAN);
    setIsScanning(true);
  }

  async function stopScan() {
    await invoke(STOP_SCAN);
    setIsScanning(false);
  }

  function toggleScan() {
    isScanning ? stopScan() : enableAndStartScan();
  }

  useEffect(() => {
    if (!isScanning) return;
    const i = setInterval(() => stopScan(), SCAN_LENGTH);
    return () => clearInterval(i);
  }, [isScanning]);

  useEffect(() => {
    const unlistenPromise = listen<FeCoreEvent>(CORE_EVENT, (event) => {
      let data;
      switch (event.payload.kind) {
        case "Scan":
          setIsScanning(event.payload.data == "Start");
          break;
        case "State":
          data = event.payload.data;
          switch (data) {
            case "Disable":
              stopScanAndDisable();
              break;
            case "EnableAndScan":
              enableAndStartScan();
              break;
            default:
              assertExhaustive(data);
          }
          break;
        default:
          assertExhaustive(event.payload);
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // Config here because I don't want the async refresh inside the Settings Dialog
  // Not sure where else to put it
  async function refreshConfig() {
    await invoke<FeVibeCheckConfig>(GET_CONFIG)
      .then((r:FeVibeCheckConfig) => setConfig(r))
      .catch(() => setConfig(null));
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
