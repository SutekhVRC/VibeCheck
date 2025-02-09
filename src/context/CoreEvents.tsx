import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { createContext, useContext, useEffect, useState } from "react";
import { toast } from "sonner";
import type { FeCoreEvent } from "../../src-tauri/bindings/FeCoreEvent";
import type { FeStateEvent } from "../../src-tauri/bindings/FeStateEvent";
import type { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import { INVOKE, LISTEN } from "../data/constants";

const SCAN_LENGTH = 10000;

type CoreEventContextProps = {
  isScanning: boolean;
  isEnabled: boolean;
  toggleIsEnabled: () => Promise<void>;
  toggleScan: () => Promise<void>;
  config: FeVibeCheckConfig | undefined;
  refreshConfig: () => Promise<void>;
};

const CoreEventContext = createContext<CoreEventContextProps>({
  isScanning: false,
  isEnabled: false,
  toggleIsEnabled: () => new Promise(() => null),
  toggleScan: () => new Promise(() => null),
  config: undefined,
  refreshConfig: () => new Promise(() => null),
});

export function useCoreEventContext() {
  const context = useContext(CoreEventContext);
  if (!context) {
    throw new Error("useCoreEventContext not within context provider");
  }
  return context;
}

export function CoreEventProvider({ children }: { children: React.ReactNode }) {
  const [isEnabled, setIsEnabled] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [config, setConfig] = useState<FeVibeCheckConfig | undefined>(
    undefined,
  );

  async function enable() {
    try {
      await invoke(INVOKE.ENABLE);
      setIsEnabled(true);
    } catch (e) {
      toast.error(`Could not enable!\n${JSON.stringify(e)}`);
    }
  }

  async function stopScanAndDisable() {
    try {
      await stopScan();
      await invoke(INVOKE.DISABLE);
      setIsEnabled(false);
    } catch (e) {
      toast.error(`Could not disable!\n${JSON.stringify(e)}`);
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
      await invoke(INVOKE.START_SCAN);
      setIsScanning(true);
    } catch (e) {
      toast.error(`Could not start scan!\n${JSON.stringify(e)}`);
    }
  }

  async function stopScan() {
    try {
      await invoke(INVOKE.STOP_SCAN);
      setIsScanning(false);
    } catch (e) {
      toast.error(`Could not stop scan!\n${JSON.stringify(e)}`);
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

  function handleStateEvent(payload: FeStateEvent): boolean {
    switch (payload) {
      case "Disable":
        stopScanAndDisable();
        return true;
      case "EnableAndScan":
        enableAndStartScan();
        return true;
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
    }
  }

  useEffect(() => {
    const unlistenPromise = listen<FeCoreEvent>(LISTEN.CORE_EVENT, (event) =>
      handleCoreEvent(event.payload),
    );

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  async function refreshConfig() {
    try {
      const config = await invoke<FeVibeCheckConfig>(INVOKE.GET_CONFIG);
      setConfig(config);
    } catch {
      setConfig(undefined);
    }
  }

  useEffect(() => {
    async function getConfig() {
      try {
        const config = await invoke<FeVibeCheckConfig>(INVOKE.GET_CONFIG);
        setConfig(config);
      } catch {
        setConfig(undefined);
      }
    }
    getConfig();
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
