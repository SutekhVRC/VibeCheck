import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
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

export default function App() {
  const [isEnabled, setIsEnabled] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [toys, setToys] = useState<FeVCToy[]>([]);

  async function getToys() {
    await invoke<null | FeVCToy[]>(GET_TOYS, {}).then((response) => {
      if (!response) return;
      const toys = Object.values(response).map((toy) => toy);
      setToys(toys);
    });
  }

  async function toggleScan() {
    await invoke(isScanning ? STOP_SCAN : START_SCAN, {}).then(() =>
      setIsScanning(!isScanning)
    );
  }

  async function toggleIsEnabled() {
    await invoke(isEnabled ? DISABLE : ENABLE, {}).then(() =>
      setIsEnabled(!isEnabled)
    );
  }

  function openSettings() {}

  useEffect(() => {
    // While scanning, check backend every second
    const intervalId = setInterval(() => {
      if (isScanning) {
        getToys();
      }
    }, 1000);
    return () => clearInterval(intervalId);
  });

  useEffect(() => {
    // Enable Vibecheck if we turn on scan
    if (isScanning && !isEnabled) {
      toggleIsEnabled();
    }
  }, [isScanning]);

  useEffect(() => {
    // Turn off scan if we disable
    if (isScanning && !isEnabled) {
      toggleScan();
    }
  }, [isEnabled]);

  return (
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
          <div key={toy.toy_id} className="item">
            <div style={{ textDecoration: "underline" }}>{toy.toy_name}</div>
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
            justifyContent: "space-around",
          }}
        >
          <div className="grad-container grad-btn-container">
            <button className="btn" type="button">
              <i className="fa fa-gear" onClick={() => openSettings()} />
            </button>
          </div>
          <div
            className={`grad-container grad-btn-container${
              isScanning ? " is-on" : ""
            }`}
          >
            <button className="btn" type="button" onClick={() => toggleScan()}>
              <i className="fa fa-wifi"></i>
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
            >
              Enable
              <i className="fa fa-play" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
