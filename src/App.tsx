import { invoke } from "@tauri-apps/api";
import { ReactNode, useEffect, useState } from "react";
import { FeVCToy } from "../src-tauri/bindings/FeVCToy";

import logo from "./assets/logo.png";
import githubLogo from "./assets/GitHub-Mark-Light-64px.png";
import discordLogo from "./assets/discord-mark-white.svg";
import {
  GET_TOYS,
  START_SCAN,
  STOP_SCAN,
  ENABLE,
  DISABLE,
} from "./data/constants";
import { percentFormat } from "./utils";

import "./App.css";
import Modal from "./components/Modal";
import Settings from "./components/Settings";

export default function App() {
  const [isEnabled, setIsEnabled] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [toys, setToys] = useState<FeVCToy[]>([]);

  const [modal, setModal] = useState<ReactNode>(null);

  async function getToys() {
    await invoke<null | FeVCToy[]>(GET_TOYS, {}).then((response) => {
      if (response) {
        setToys(Object.values(response));
      } else {
        setToys([]);
      }
    });
  }

  async function toggleIsScanning() {
    if (isScanning) {
      await invoke(STOP_SCAN, {}).then(() => setIsScanning(false));
    } else {
      if (!isEnabled) {
        // Enable Vibecheck if we turn on scan
        toggleIsEnabled();
      }
      await invoke(START_SCAN, {}).then(() => setIsScanning(true));
    }
  }

  async function toggleIsEnabled() {
    if (isEnabled) {
      if (isScanning) {
        // Turn off scan if we disable Vibecheck
        toggleIsScanning();
      }
      await invoke(DISABLE, {}).then(() => setIsEnabled(false));
      getToys(); // Clear toy list after we disable
    } else {
      await invoke(ENABLE, {}).then(() => setIsEnabled(true));
    }
  }

  useEffect(() => {
    if (isScanning) {
      // While scanning, check backend every second
      const interval = setInterval(() => {
        getToys();
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
    <>
      {modal}
      <div style={{ display: "flex", justifyContent: "center" }}>
        <div className="main-container">
          <div className="header">
            <div className="grad-container">
              <img src={logo} />
            </div>
            Beta 0.2.0
            <img src={discordLogo} />
            <img src={githubLogo} />
          </div>
          <div className="toys-container">
            <h1 className="grad-text">Connected toys</h1>
            {toys.map((toy) => (
              <div
                key={toy.toy_id}
                className="toy"
                onClick={() =>
                  setModal(
                    <Modal onClose={() => setModal(null)}>
                      <pre>{JSON.stringify(toy, null, 2)}</pre>
                    </Modal>
                  )
                }
              >
                <div style={{ textDecoration: "underline" }}>
                  {toy.toy_name}
                </div>
                <div
                  style={{
                    color: `rgb(${(1 - toy.battery_level) * 255},${
                      toy.battery_level * 255
                    },0)`,
                  }}
                >
                  {percentFormat.format(toy.battery_level)}
                </div>
              </div>
            ))}
            <div
              style={{
                display: "flex",
                justifyContent: "space-evenly",
              }}
            >
              <div className="grad-container grad-btn-container">
                <button
                  className="btn"
                  type="button"
                  onClick={() =>
                    setModal(
                      <Modal
                        children={<Settings />}
                        onClose={() => setModal(null)}
                      />
                    )
                  }
                >
                  <i className="fa fa-gear" />
                </button>
              </div>
              <div
                className={`grad-container grad-btn-container${
                  isScanning ? " is-on" : ""
                }`}
              >
                <button
                  className="btn"
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
                  className="btn"
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
          </div>
        </div>
      </div>
    </>
  );
}
