import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import { useToys } from "../context/ToysContext";

import {
  DISABLE,
  ENABLE,
  SCAN_LENGTH,
  START_SCAN,
  STOP_SCAN,
} from "../data/constants";
import SettingsModal from "./SettingsModal";

export default function () {
  const [isEnabled, setIsEnabled] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [settingsIsOpen, setSettingsIsOpen] = useState(false);

  const { refetchToys } = useToys();

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
      refetchToys(); // Clear toy list after we disable
    } else {
      await invoke(ENABLE).then(() => setIsEnabled(true));
    }
  }

  useEffect(() => {
    if (isScanning) {
      // Turn off scan after x ms
      const timeout = setTimeout(() => {
        toggleIsScanning();
      }, SCAN_LENGTH);

      return () => {
        clearTimeout(timeout);
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
          show={settingsIsOpen}
          onHide={() => setSettingsIsOpen(false)}
        />
      </div>
      <div
        className={
          isScanning
            ? "grad-container grad-btn-container is-on"
            : "grad-container grad-btn-container"
        }
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
        className={
          isEnabled
            ? "grad-container grad-btn-container is-on"
            : "grad-container grad-btn-container"
        }
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
