import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import Image from "next/image";
import reactLogo from "../assets/react.svg";
import tauriLogo from "../assets/tauri.svg";
import nextLogo from "../assets/next.svg";

type FeatureLevels = {
  idle_level: number;
  maximumLevel: number;
  minimumLevel: number;
  smoothRate: number;
}

type ToyFeatureMapWrap = {
  features: ToyFeatureMap[];
}

type ToyFeatureMap = {
  featureEnabled: boolean;
  featureIndex: number;
  featureLevels: FeatureLevels;
  featureType: string;
  oscParameter: string;
  smoothEnabled: boolean;
  smoothEntries: number[];
}

type VibeCheckToy = {
  toyName: string;
  batteryLevel: number;
  featureMap: ToyFeatureMapWrap;
  toyConnected: boolean;
  toyId: number;
}

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [toys, setToys] = useState([]);
  //var toys = [];

  async function getToys() {

    await invoke("get_toys", {}).then((response: VibeCheckToy[]) => {
      setToys(response);
      //console.log(response);
    });
  }

  async function startScan() {
    await invoke("vibecheck_start_bt_scan", {});
  }

  async function stopScan() {
    await invoke("vibecheck_stop_bt_scan", {});
  }

  async function enable() {
    await invoke("vibecheck_enable", {});
  }

  async function disable() {
    await invoke("vibecheck_disable", {});
  }


  function listToys() {
    if (!toys) return;
    let names = [];
    for (const [k, v] of Object.entries(toys)) {
      console.log(v);
      names.push(<li>{v.toy_name}</li>);
    }
    return names;
  }

  return (

    <div className="container">

      <h1> Toys </h1>
      <div className="row">
      <ul>
          {listToys()}
        </ul>
      </div>

    <div className="row">
      <div>
        <button type="button" onClick={() => enable()}>
          Enable VibeCheck
        </button>
    </div>
  </div>

  
      <div className="row">
        <div>
          <button type="button" onClick={() => disable()}>
            Disable VibeCheck
          </button>
      </div>
    </div>

  
      <div className="row">
        <div>
          <button type="button" onClick={() => startScan()}>
            Start Scanning
          </button>
      </div>
    </div>


      <div className="row">
        <div>
          <button type="button" onClick={() => stopScan()}>
            Stop Scanning
          </button>
        </div>
      </div>
      
      <div className="row">
        <div>
          <button type="button" onClick={() => getToys()}>
            Get Toys
          </button>
        </div>
      </div>
    </div>
  );
}

export default App;

/*
<h1>Welcome to Tauri!</h1>

      <div className="row">
        <span className="logos">
          <a href="https://nextjs.org" target="_blank">
            <Image
              width={144}
              height={144}
              src={nextLogo}
              className="logo next"
              alt="Next logo"
            />
          </a>
        </span>
        <span className="logos">
          <a href="https://tauri.app" target="_blank">
            <Image
              width={144}
              height={144}
              src={tauriLogo}
              className="logo tauri"
              alt="Tauri logo"
            />
          </a>
        </span>
        <span className="logos">
          <a href="https://reactjs.org" target="_blank">
            <Image
              width={144}
              height={144}
              src={reactLogo}
              className="logo react"
              alt="React logo"
            />
          </a>
        </span>
      </div>

      <p>Click on the Tauri, Next, and React logos to learn more.</p>

      <div className="row">
        <div>
          <input
            id="greet-input"
            onChange={(e) => setName(e.currentTarget.value)}
            placeholder="Enter a name..."
          />
          <button type="button" onClick={() => greet()}>
            Greet
          </button>
        </div>
      </div>
*/