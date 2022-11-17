import { invoke } from "@tauri-apps/api";
import { useState } from "react";
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
  battery_lvel: number;
  param_feature_map: ToyFeatureMapWrap;
  toy_connected: boolean;
  toy_id: number;
};

type GetToysResponse = null | {
  [key: number]: VibeCheckToy;
};

export default function App() {
  const [enabled, setEnabled] = useState(false);
  const [toys, setToys] = useState<VibeCheckToy[]>([]);

  async function getToys() {
    await invoke<GetToysResponse>("get_toys", {}).then((response) => {
      if (!response) return;
      const toys = Object.values(response).map((toy) => toy);
      setToys(toys);
    });
  }

  async function startScan() {
    await invoke("vibecheck_start_bt_scan", {});
  }

  async function stopScan() {
    await invoke("vibecheck_stop_bt_scan", {});
  }

  async function enable() {
    if (enabled) return;
    await invoke("vibecheck_enable", {}).then(() => setEnabled(true));
  }

  async function disable() {
    if (!enabled) return;
    await invoke("vibecheck_disable", {}).then(() => setEnabled(false));
  }

  return (
    <div>
      <h1> Toys </h1>
      <div>
        {toys.map((toy) => (
          <li key={toy.toy_id}>{toy.toy_name}</li>
        ))}
      </div>

      <div>
        <div>
          <button type="button" onClick={() => enable()}>
            Enable VibeCheck
          </button>
        </div>
      </div>

      <div>
        <div>
          <button type="button" onClick={() => disable()}>
            Disable VibeCheck
          </button>
        </div>
      </div>

      <div>
        <div>
          <button type="button" onClick={() => startScan()}>
            Start Scanning
          </button>
        </div>
      </div>

      <div>
        <div>
          <button type="button" onClick={() => stopScan()}>
            Stop Scanning
          </button>
        </div>
      </div>

      <div>
        <div>
          <button type="button" onClick={() => getToys()}>
            Get Toys
          </button>
        </div>
      </div>
    </div>
  );
}
