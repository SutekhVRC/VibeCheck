import { invoke } from "@tauri-apps/api";
import { useState } from "react";
import { FeFeatureParamMap } from "./bindings/FeFeatureParamMap";
import { FeLevelTweaks } from "./bindings/FeLevelTweaks";
import { FeVCFeatureType } from "./bindings/FeVCFeatureType";
import { FeVCToy } from "./bindings/FeVCToy";
import { FeVCToyFeature } from "./bindings/FeVCToyFeature";
import { FeVibeCheckConfig } from "./bindings/FeVibeCheckConfig";
import { FeOSCNetworking } from "./bindings/FeOSCNetworking";

import logo from "./assets/logo.png";
import discordLogo from "./assets/discord-mark-black.svg";
import githubLogo from "./assets/GitHub-Mark-120px-plus.png";
import "./App.css";

type FeatureLevels = {
  idle_level: number;
  maximum_level: number;
  minimum_level: number;
  smooth_rate: number;
};

type ToyFeatureMapWrap = {
  features: ToyFeatureMap[];
};

type ToyFeatureMap = {
  feature_enabled: boolean;
  feature_index: number;
  feature_levels: FeatureLevels;
  feature_type: string;
  osc_parameter: string;
  smooth_enabled: boolean;
  smooth_entries: number[];
};

type VibeCheckToy = {
  toy_name: string;
  battery_level: number;
  param_feature_map: ToyFeatureMapWrap;
  toy_connected: boolean;
  toy_id: number;
};

type GetToysResponse = null | {
  [key: number]: VibeCheckToy;
};

const percentFormat = new Intl.NumberFormat("en-US", {
  style: "percent",
  minimumFractionDigits: 0,
  maximumFractionDigits: 0,
});

export default function App() {
  const [isEnabled, setIsEnabled] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [toys, setToys] = useState<VibeCheckToy[]>([]);

  async function getToys() {
    // Does this automatically enable?
    await invoke<GetToysResponse>("get_toys", {}).then((response) => {
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
        <img src={githubLogo} style={{ maxHeight: "50px" }} />
      </div>
      <h1 className="grad-text">Connected toys</h1>
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          backgroundColor: "rgb(25,25,25)",
        }}
      >
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
      </div>
      <div>
        <button type="button" onClick={() => getToys()}>
          Get Toys
        </button>
        <button type="button" onClick={() => toggleScan()}>
          {isEnabled ? "Stop Scanning" : "Start Scanning"}
        </button>
        <button type="button" onClick={() => toggleIsEnabled()}>
          {isEnabled ? "Disable" : "Enable"}
        </button>
      </div>
    </>
  );
}
