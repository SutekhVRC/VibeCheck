import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import { FeVCToy } from "../src-tauri/bindings/FeVCToy";
import { FeVibeCheckConfig } from "../src-tauri/bindings/FeVibeCheckConfig";
import { open } from "@tauri-apps/api/shell";

import logo from "./assets/logo.png";
import githubLogo from "./assets/GitHub-Mark-Light-64px.png";
import discordLogo from "./assets/discord-mark-white.svg";
import {
  GET_TOYS,
  START_SCAN,
  STOP_SCAN,
  ENABLE,
  DISABLE,
  VERSION,
  GET_CONFIG,
} from "./data/constants";
import { percent } from "./utils";

import "./App.css";
import Settings from "./components/Settings";
import ToySettings from "./components/ToySettings";
import Accordion from "react-bootstrap/Accordion";

const version = await invoke<string>(VERSION).then((r) => {
  return r.replace(/-/g, " ");
});

export default function App() {
  const [isEnabled, setIsEnabled] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [toys, setToys] = useState<FeVCToy[]>([]);

  const [settingsIsOpen, setSettingsIsOpen] = useState(false);
  const [settings, setSettings] = useState<null | FeVibeCheckConfig>(null);
  async function getConfig() {
    setSettings(await invoke<FeVibeCheckConfig>(GET_CONFIG));
  }
  useEffect(() => {
    getConfig();
  }, []);

  async function getToys() {
    await invoke<null | { [key: number]: FeVCToy }>(GET_TOYS).then(
      (response) => {
        if (response) {
          setToys(Object.values(response));
        } else {
          setToys([]);
        }
      }
    );
  }

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
      getToys(); // Clear toy list after we disable
    } else {
      await invoke(ENABLE).then(() => setIsEnabled(true));
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
      <div style={{ display: "flex", justifyContent: "center" }}>
        <div className="main-container">
          <div className="header">
            <div className="grad-container">
              <img className="logo" src={logo} />
            </div>
            {version}
            <img
              className="extern-logo"
              src={discordLogo}
              onClick={() => open("https://discord.gg/g6kUFtMtpw")}
            />
            <img
              className="extern-logo"
              src={githubLogo}
              onClick={() => open("https://github.com/SutekhVRC/VibeCheck")}
            />
          </div>
          <div className="toys-container">
            <h1 className="grad-text">Connected toys</h1>
            {toys.map((toy) => (
              <div key={toy.toy_id} className="toy-container">
                <div className="toy">
                  <div>{toy.toy_name}</div>
                  <div
                    style={{
                      color: `hsl(${toy.battery_level * 120}, 100%, 50%)`,
                    }}
                  >
                    {percent.format(toy.battery_level)}
                  </div>
                </div>
                <Accordion>
                  {toy.features.map((feature) => (
                    <Accordion.Item
                      eventKey={feature.feature_index.toString()}
                      key={feature.feature_index}
                    >
                      <Accordion.Header>
                        {`${feature.feature_type} ${feature.feature_index}`}
                      </Accordion.Header>
                      <Accordion.Body>
                        <ToySettings {...feature} />
                      </Accordion.Body>
                    </Accordion.Item>
                  ))}
                </Accordion>
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
                  className="btn-custom"
                  type="button"
                  onClick={() => setSettingsIsOpen(true)}
                >
                  <i className="fa fa-gear" />
                </button>
                <Settings
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
          </div>
        </div>
      </div>
    </>
  );
}
