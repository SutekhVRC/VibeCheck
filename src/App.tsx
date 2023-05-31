import { useToys } from "./hooks/useToys";
import { Footer } from "./features/Footer";
import VibecheckLogo from "./assets/VibeCheck_logo.png";
import cryingAnimeGirl from "./assets/menhera_chan.gif";
import Toy from "./features/Toy";
import { AnimatePresence } from "framer-motion";
import Button from "./layout/Button";
import { useCoreEventContext } from "./context/CoreEventContext";
import "./App.css";
import Loading from "./components/Loading";
import { useState } from "react";
import { FeVCToy } from "../src-tauri/bindings/FeVCToy";
import classNames from "classnames";

export default function App() {
  const [selectedToy, setSelectedToy] = useState<FeVCToy | null>(null);
  const { onlineToys, offlineToys } = useToys();
  const toys: FeVCToy[] = [];
  Object.values(onlineToys).forEach((t) => toys.push(t));
  Object.values(offlineToys).forEach((t) => toys.push(t));
  const { isScanning, toggleScan } = useCoreEventContext();

  return (
    <div className="w-screen h-screen p-4">
      <div className="grid grid-cols-[1fr,_4fr] h-[calc(100%-40px)] gap-3">
        <div className="flex flex-col gap-2">
          <img className="h-16 object-contain" src={VibecheckLogo} />
          <div className=" bg-gray-800 rounded-md justify-between flex flex-col max-h-fit flex-grow">
            {toys.length === 0 ? (
              <div className="flex-grow flex justify-center">
                <div className="flex flex-col justify-center items-center -mt-20">
                  <img src={cryingAnimeGirl} />
                  <div>No Toys</div>
                </div>
              </div>
            ) : (
              <div className="flex flex-col overflow-y-scroll pl-2 scrollbar whitespace-nowrap">
                <AnimatePresence>
                  {Object.values(toys).map((toy) => (
                    <button
                      key={`${toy.toy_name} ${toy.sub_id}`}
                      onClick={() => {
                        setSelectedToy(toy);
                      }}
                      className={classNames(
                        toy.toy_name == selectedToy?.toy_name &&
                          toy.sub_id == selectedToy?.sub_id &&
                          "outline",
                        toy.toy_connected ? "text-gray-200" : "text-gray-500",
                        "bg-gray-700 rounded-md p-2 m-2 hover:bg-cyan-600 outline-2 outline-cyan-400"
                      )}
                    >
                      {toy.toy_name}
                    </button>
                  ))}
                </AnimatePresence>
              </div>
            )}
            <div>
              <Button onClick={toggleScan}>
                {isScanning ? (
                  <div className="flex justify-center">
                    <div>Scanning</div>
                    <Loading />
                  </div>
                ) : (
                  <div>Search for toys</div>
                )}
              </Button>
            </div>
          </div>
        </div>
        <div className="bg-gray-800 rounded-lg">
          <div className="flex justify-between p-4">
            {selectedToy != null && (
              <Toy
                toy={selectedToy}
                key={`${selectedToy.toy_name} ${selectedToy.sub_id}`}
              />
            )}
          </div>
        </div>
      </div>
      <div className="m-2">
        <Footer />
      </div>
    </div>
  );
}
