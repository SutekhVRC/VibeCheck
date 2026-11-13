import type { FeCoreEvent } from "@bindings/FeCoreEvent";
import type { FeSafetyEvent } from "@bindings/FeSafetyEvent";
import { FeStateEvent } from "@bindings/FeStateEvent";
import type { FeVibeCheckConfig } from "@bindings/FeVibeCheckConfig";
import type { FrontendError } from "@bindings/FrontendError";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createContext, useContext, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { INVOKE, LISTEN } from "../data/constants";
import { assertExhaustive } from "../utils";

type CoreEventContextProps = {
  isScanning: boolean;
  isEnabled: boolean;
  toggleIsEnabled: () => Promise<void>;
  toggleScan: () => Promise<void>;
  config: FeVibeCheckConfig | undefined;
  /** False until the first attempt to load the config has finished */
  configLoaded: boolean;
  refreshConfig: () => Promise<void>;
  /**
   * Changes some config settings and saves them to Config.json. The change
   * shows straight away and is undone if saving fails.
   */
  saveConfig: (changes: Partial<FeVibeCheckConfig>) => Promise<void>;
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
  configLoaded: false,
  refreshConfig: () => new Promise(() => null),
  saveConfig: () => new Promise(() => null),
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
  const [config, setConfigState] = useState<FeVibeCheckConfig | undefined>(
    undefined,
  );
  const [configLoaded, setConfigLoaded] = useState(false);
  // The latest config, for saves that start before a re-render
  const configRef = useRef<FeVibeCheckConfig | undefined>(undefined);

  function setConfig(next: FeVibeCheckConfig | undefined) {
    configRef.current = next;
    setConfigState(next);
  }

  async function saveConfig(changes: Partial<FeVibeCheckConfig>) {
    const previous = configRef.current;
    if (!previous) return;
    const next = { ...previous, ...changes };
    setConfig(next);
    try {
      await invoke(INVOKE.SET_CONFIG, { feVcConfig: next });
    } catch (e) {
      setConfig(previous);
      toast.error(`Could not save settings!\n${JSON.stringify(e)}`);
    }
  }

  // Saved in Config.json, so it survives updates
  const autoEnableOnScan = config?.auto_enable_on_scan ?? false;
  function setAutoEnableOnScan(value: boolean) {
    saveConfig({ auto_enable_on_scan: value });
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
      } finally {
        setConfigLoaded(true);
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
        configLoaded,
        refreshConfig,
        saveConfig,
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
