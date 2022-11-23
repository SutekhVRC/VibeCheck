import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";

import { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import {
  DISABLE,
  ENABLE,
  GET_CONFIG,
  START_SCAN,
  STOP_SCAN,
} from "../data/constants";
import SettingsModal from "./SettingsModal";

export default function (props: { getToys: () => void }) {
  const [isEnabled, setIsEnabled] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [settingsIsOpen, setSettingsIsOpen] = useState(false);
  const [settings, setSettings] = useState<null | FeVibeCheckConfig>(null);

  async function getConfig() {
    setSettings(await invoke<FeVibeCheckConfig>(GET_CONFIG));
  }
  useEffect(() => {
    getConfig();
  }, []);

  async function toggleIsScanning() {
    if (isScanning) {
      await invoke(STOP_SCAN).then(() => setIsScanning(false));
    } else {
      if (!isEnabled) {
        // Enable Vibecheck if we turn on scan
        toggleIsEnabled();
      }
      await invoke(START_SCAN).then(() => setIsScanning(true));
    }
  }

  async function toggleIsEnabled() {
    if (isEnabled) {
      if (isScanning) {
        // Turn off scan if we disable Vibecheck
        toggleIsScanning();
      }
      await invoke(DISABLE).then(() => setIsEnabled(false));
      props.getToys(); // Clear toy list after we disable
    } else {
      await invoke(ENABLE).then(() => setIsEnabled(true));
    }
  }

  useEffect(() => {
    if (isScanning) {
      // While scanning, check backend every second
      const interval = setInterval(() => {
        props.getToys();
      }, 1000);

      // Turn off scan after 10 seconds
      const timeout = setTimeout(() => {
        toggleIsScanning();
      }, 10000);

      return () => {
        clearTimeout(timeout);
        clearInterval(interval);
      };
    }
  }, [isScanning]);

  return (
    <div className="footer">
      <div className="grad-container grad-btn-container">
        <button
          className="btn-custom"
          type="button"
          onClick={() => setSettingsIsOpen(true)}
        >
          <i className="fa fa-gear" />
        </button>
        <SettingsModal
          settings={settings}
          show={settingsIsOpen}
          onHide={() => setSettingsIsOpen(false)}
          onSave={() => getConfig()} // Refresh settings from backend
        />
      </div>
      <div
        className={`grad-container grad-btn-container${
          isScanning ? " is-on" : ""
        }`}
      >
        <button
          className="btn-custom"
          type="button"
          onClick={() => toggleIsScanning()}
        >
          {isScanning ? (
            <i className="fa fa-eye-slash" />
          ) : (
            <i className="fa fa-eye" />
          )}
        </button>
      </div>
      <div
        className={`grad-container grad-btn-container${
          isEnabled ? " is-on" : ""
        }`}
      >
        <button
          className="btn-custom"
          type="button"
          onClick={() => toggleIsEnabled()}
          style={{
            display: "flex",
            alignItems: "center",
            paddingLeft: ".5rem",
            paddingRight: ".5rem",
          }}
        >
          <div
            style={{
              fontSize: "1.25rem",
              paddingRight: ".5rem",
            }}
          >
            {isEnabled ? "Disable" : "Enable"}
          </div>
          <i className="fa fa-play" />
        </button>
      </div>
    </div>
  );
}
