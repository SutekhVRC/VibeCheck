import { invoke } from "@tauri-apps/api";
import { useState } from "react";

import { FeVCToy } from "../src-tauri/bindings/FeVCToy";
import { GET_TOYS } from "./data/constants";
import Header from "./components/Header";
import Toy from "./components/Toy";
import Footer from "./components/Footer";
import "./App.css";

export default function App() {
  const [toys, setToys] = useState<FeVCToy[]>([]);

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

  return (
    <div style={{ display: "flex", justifyContent: "center" }}>
      <div className="main-container">
        <Header />
        <div className="toys-container">
          <h1 className="grad-text">Connected toys</h1>
          {toys.length > 0 ? (
            toys.map((toy) => <Toy toy={toy} key={toy.toy_id} />)
          ) : (
            <div>None</div>
          )}
        </div>
        <Footer getToys={getToys} />
      </div>
    </div>
  );
}
