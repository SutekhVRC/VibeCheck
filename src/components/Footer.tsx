import { useState } from "react";
import { useCoreEventContext } from "../context/CoreEventContext";
import SettingsModal from "./SettingsModal";

export default function () {
  const [settingsIsOpen, setSettingsIsOpen] = useState(false);

  const { isScanning, isEnabled, toggleIsEnabled, startScan } =
    useCoreEventContext();

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
          onClick={() => startScan()}
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
