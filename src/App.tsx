import { invoke } from '@tauri-apps/api'
import { useState } from 'react'
import reactLogo from './assets/react.svg'
import './App.css'


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
  const [count, setCount] = useState(0)
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [toys, setToys] = useState<VibeCheckToy[]>([]);
  //var toys = [];

  async function getToys() {

    await invoke("get_toys", {}).then((response) => {
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
      //console.log(v);
      names.push(<li key={v.toyId}>{v.toy_name}</li>);
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

/*
  return (
    <div className="App">
      <div>
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo" alt="Vite logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </div>
  )*/
}

export default App
