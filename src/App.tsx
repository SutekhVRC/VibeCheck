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

export default function App() {
  const { toys } = useToys();
  const { isScanning, toggleScan } = useCoreEventContext();

  return (
    <div className="w-screen h-screen p-4">
      <div className="flex justify-center pb-2">
        <img className="h-20 object-contain" src={VibecheckLogo} />
      </div>
      <div className="h-[calc(100vh-160px)] flex justify-center">
        <div className="flex flex-col justify-between gap-2 text-lg rounded-lg p-3 bg-zinc-800 flex-grow max-w-xl">
          {Object.keys(toys).length == 0 ? (
            <div className="flex-grow flex justify-center">
              <div className="flex flex-col justify-center items-center -mt-20">
                <img src={cryingAnimeGirl} />
                <div>No Toys</div>
              </div>
            </div>
          ) : (
            <div className="overflow-y-scroll pl-2 scrollbar">
              <AnimatePresence>
                {Object.values(toys).map((toy) => (
                  <Toy toy={toy} key={`${toy.toy_name} ${toy.toy_id}`} />
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
      <div className="h-14 flex justify-center items-center">
        <Footer />
      </div>
    </div>
  );
}
