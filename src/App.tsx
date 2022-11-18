import { invoke } from "@tauri-apps/api";
import { useState } from "react";
import { FeVCToy } from "../src-tauri/bindings/FeVCToy";

import logo from "./assets/logo.png";
import discordLogo from "./assets/discord-mark-black.svg";
import "./App.css";
import "font-awesome/css/font-awesome.min.css";

const percentFormat = new Intl.NumberFormat("en-US", {
  style: "percent",
  minimumFractionDigits: 0,
  maximumFractionDigits: 0,
});

export default function App() {
  const [isEnabled, setIsEnabled] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [toys, setToys] = useState<FeVCToy[]>([]);

  async function getToys() {
    // Does this automatically enable?
    await invoke<null | FeVCToy[]>("get_toys", {}).then((response) => {
      if (!response) return;
      const toys = Object.values(response).map((toy) => toy);
      setToys(toys);
    });
  }

  async function toggleScan() {
    // Maybe need catch if frontend state becomes unlinked?
    if (isScanning) {
      await invoke("vibecheck_stop_bt_scan", {}).then(() =>
        setIsScanning(false)
      );
    } else {
      await invoke("vibecheck_start_bt_scan", {}).then(() =>
        setIsScanning(true)
      );
    }
  }

  async function toggleIsEnabled() {
    // Maybe need catch if frontend state becomes unlinked?
    if (isEnabled) {
      await invoke("vibecheck_disable", {}).then(() => setIsEnabled(false));
    } else {
      await invoke("vibecheck_enable", {}).then(() => setIsEnabled(true));
    }
  }

  return (
    <>
      <div style={{ display: "flex", justifyContent: "center" }}>
        <img src={logo} style={{ maxHeight: "50px" }} />
        Beta 0.2.0
        <img src={discordLogo} style={{ maxHeight: "50px" }} />
        <i className="fa fa-github grad-icon" />
      </div>
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          backgroundColor: "rgb(25,25,25)",
        }}
      >
        <h1 className="grad-text">Connected toys</h1>
        {toys.map((toy) => (
          <div
            key={toy.toy_id}
            style={{
              display: "flex",
              justifyContent: "space-between",
              backgroundColor: "rgb(50,50,50)",
              minWidth: "400px",
            }}
          >
            <div style={{ textDecoration: "underline" }}>{toy.toy_name}</div>
            <div style={{ color: "rgb(0,255,0)" }}>
              {percentFormat.format(toy.battery_level)}
            </div>
          </div>
        ))}
        <div style={{ display: "flex", justifyContent: "space-around" }}>
          <i className="fa fa-gear grad-icon" onClick={() => getToys()} />
          <i className="fa fa-wifi grad-icon" onClick={() => toggleScan()} />
          <i
            className="fa fa-play grad-icon"
            onClick={() => toggleIsEnabled()}
          />
        </div>
      </div>
    </>
  );
}
