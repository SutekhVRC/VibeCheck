import type { FeCoreEvent } from "@bindings/FeCoreEvent";
import type { FeSafetyEvent } from "@bindings/FeSafetyEvent";
import { FeStateEvent } from "@bindings/FeStateEvent";
import type { FeVibeCheckConfig } from "@bindings/FeVibeCheckConfig";
import type { FrontendError } from "@bindings/FrontendError";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createContext, useContext, useEffect, useState } from "react";
import { toast } from "sonner";
import { INVOKE, LISTEN } from "../data/constants";
import { assertExhaustive } from "../utils";

/*
 * Frontend-only preference (saved in localStorage, not the backend config):
 * whether pressing the scan button turns VibeCheck on first.
 */
const AUTO_ENABLE_ON_SCAN_KEY = "vibecheck-auto-enable-on-scan";

type CoreEventContextProps = {
  isScanning: boolean;
  isEnabled: boolean;
  toggleIsEnabled: () => Promise<void>;
  toggleScan: () => Promise<void>;
  config: FeVibeCheckConfig | undefined;
  refreshConfig: () => Promise<void>;
  /** Turn the app on automatically when the scan button is pressed */
  autoEnableOnScan: boolean;
  setAutoEnableOnScan: (value: boolean) => void;
  /** Panic stop: every toy stopped until resumed */
  isPanicked: boolean;
  panicStop: () => Promise<void>;
  resumeFromPanic: () => Promise<void>;
};

const CoreEventContext = createContext<CoreEventContextProps>({
  isScanning: false,
  isEnabled: false,
  toggleIsEnabled: () => new Promise(() => null),
  toggleScan: () => new Promise(() => null),
  config: undefined,
  refreshConfig: () => new Promise(() => null),
  autoEnableOnScan: false,
  setAutoEnableOnScan: () => null,
  isPanicked: false,
  panicStop: () => new Promise(() => null),
  resumeFromPanic: () => new Promise(() => null),
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
  // Also changed by the hotkey and OSC API, so the backend's events are the
  // source of truth; the invokes below don't set it themselves
  const [isPanicked, setIsPanicked] = useState(false);
  const [config, setConfig] = useState<FeVibeCheckConfig | undefined>(
    undefined,
  );
  const [autoEnableOnScan, setAutoEnableOnScanState] = useState(
    () => localStorage.getItem(AUTO_ENABLE_ON_SCAN_KEY) === "true",
  );

  function setAutoEnableOnScan(value: boolean) {
    localStorage.setItem(AUTO_ENABLE_ON_SCAN_KEY, String(value));
    setAutoEnableOnScanState(value);
  }

  /** Turns the app on; returns whether it worked */
  async function enable() {
    try {
      await invoke(INVOKE.ENABLE);
      setIsEnabled(true);
      return true;
    } catch (e) {
      toast.error(`Could not enable!\n${JSON.stringify(e)}`);
      return false;
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

  async function startScan() {
    try {
      await invoke(INVOKE.START_SCAN);
      setIsScanning(true);
    } catch (e) {
      toast.error(`Could not start scan!\n${JSON.stringify(e)}`);
    }
  }

  // Used when VRChat asks VibeCheck to turn on and scan (avatar parameter)
  async function enableAndStartScan() {
    if (await enable()) await startScan();
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
    } else if (isEnabled) {
      await startScan();
    } else if (autoEnableOnScan) {
      await enableAndStartScan();
    } else {
      // The backend would start scanning even while the app is off, so
      // don't scan until VibeCheck has been turned on.
      toast.info(
        "Turn on VibeCheck first\nOr switch on \u201cTurn on when scanning\u201d in Settings",
      );
    }
  }

  // The backend stops scans after the configured scan length and sends a
  // Scan Stop event (handleCoreEvent), so there's no timer here

  function handleStateEvent(payload: FeStateEvent) {
    switch (payload) {
      case "Disable":
        stopScanAndDisable();
        break;
      case "EnableAndScan":
        enableAndStartScan();
        break;
      // From the tray menu
      case "Enable":
        enable();
        break;
      case "Scan":
        startScan();
        break;
      default:
        assertExhaustive(payload);
    }
  }

  function handleSafetyEvent(payload: FeSafetyEvent) {
    switch (payload) {
      case "Panicked":
        setIsPanicked(true);
        break;
      case "Resumed":
        setIsPanicked(false);
        break;
      default:
        assertExhaustive(payload);
    }
  }

  async function panicStop() {
    try {
      await invoke(INVOKE.PANIC_STOP);
    } catch (e) {
      toast.error(`Could not stop toys!\n${JSON.stringify(e)}`);
    }
  }

  async function resumeFromPanic() {
    try {
      await invoke(INVOKE.PANIC_RESUME);
    } catch (e) {
      toast.error(`Could not resume!\n${JSON.stringify(e)}`);
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
      case "Safety":
        handleSafetyEvent(payload.data);
        break;
      default:
        assertExhaustive(payload);
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

  // Errors reported by the backend's error signal handler
  useEffect(() => {
    const unlistenPromise = listen<FrontendError>(LISTEN.ERROR, (event) => {
      const payload = event.payload;
      switch (payload.kind) {
        case "Error":
          toast.error(payload.data);
          break;
        case "Warning":
          toast.warning(payload.data);
          break;
        case "Info":
          toast.info(payload.data);
          break;
        default:
          assertExhaustive(payload);
      }
    });

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

  // A panic may have happened before this window loaded
  useEffect(() => {
    invoke<boolean>(INVOKE.GET_PANIC_STATE)
      .then(setIsPanicked)
      .catch(() => null);
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
        autoEnableOnScan,
        setAutoEnableOnScan,
        isPanicked,
        panicStop,
        resumeFromPanic,
      }}
    >
      {children}
    </CoreEventContext.Provider>
  );
}
