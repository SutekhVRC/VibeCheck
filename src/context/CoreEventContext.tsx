import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import {
  createContext,
  ReactNode,
  useContext,
  useEffect,
  useState,
} from "react";
import { FeCoreEvent } from "../../src-tauri/bindings/FeCoreEvent";
import {
  CORE_EVENT,
  DISABLE,
  ENABLE,
  SCAN_LENGTH,
  START_SCAN,
  STOP_SCAN,
} from "../data/constants";
import { assertExhaustive } from "../utils";

type CoreContextProps = {
  isScanning: boolean;
  isEnabled: boolean;
  toggleIsEnabled: () => void;
  startScan: () => void;
};
const INITIAL_CORE_STATE: CoreContextProps = {
  isScanning: false,
  isEnabled: false,
  toggleIsEnabled: () => null,
  startScan: () => null,
};
const CoreEventContext = createContext<CoreContextProps>(INITIAL_CORE_STATE);

export function useCoreEventContext() {
  return useContext(CoreEventContext);
}

export function CoreEventProvider({ children }: { children: ReactNode }) {
  const [isEnabled, setIsEnabled] = useState(INITIAL_CORE_STATE.isEnabled);
  const [isScanning, setIsScanning] = useState(INITIAL_CORE_STATE.isScanning);

  async function toggleIsEnabled() {
    if (isEnabled) {
      await invoke(DISABLE)
        .then(() => setIsEnabled(false))
        .catch(() => setIsEnabled(false) /* Failed, already disabled */);
    } else {
      await invoke(ENABLE)
        .then(() => setIsEnabled(true))
        .catch(() => setIsEnabled(true) /* Failed, already enabled */);
    }
  }

  async function startScan() {
    if (!isEnabled) {
      toggleIsEnabled();
    }
    await invoke(START_SCAN);
    setIsScanning(true);
  }

  async function stopScan() {
    await invoke(STOP_SCAN);
    setIsScanning(false);
  }

  useEffect(() => {
    if (!isScanning) return;
    const i = setInterval(() => stopScan(), SCAN_LENGTH);
    return () => clearInterval(i);
  }, [isScanning]);

  useEffect(() => {
    const unlistenPromise = listen<FeCoreEvent>(CORE_EVENT, (event) => {
      switch (event.payload.kind) {
        case "Scan":
          setIsScanning(event.payload.data == "Start");
          break;
        default:
        // TODO add assert when > 1 objects unioned in FeCoreEvent
        // assertExhaustive(event.payload);
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  return (
    <CoreEventContext.Provider
      value={{ isScanning, isEnabled, toggleIsEnabled, startScan }}
    >
      {children}
    </CoreEventContext.Provider>
  );
}
