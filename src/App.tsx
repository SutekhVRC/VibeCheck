import { useToys } from "./hooks/useToys";
import { Footer } from "./features/Footer";
import VibecheckLogo from "./assets/VibeCheck_logo.png";
import cryingAnimeGirl from "./assets/menhera_chan.gif";
import ScanButton from "./components/ScanButton";
import Toy from "./features/Toy";
import { AnimatePresence } from "framer-motion";
import "./App.css";

export default function App() {
  const { toys } = useToys();

  return (
    <>
      <div className="flex justify-center h-20 mb-4">
        <img src={VibecheckLogo} />
      </div>
      <div className="flex-col justify-between items-stretch text-lg rounded-lg p-4 bg-zinc-800 h-[600px]">
        {Object.keys(toys).length == 0 ? (
          <div className="flex justify-center items-center">
            No Toys
            <img src={cryingAnimeGirl} />
          </div>
        ) : (
          <div className="overflow-y-scroll pl-2 scrollbar pt-2 pb-2 max-h-[520px]">
            <AnimatePresence>
              {Object.values(toys).map((toy) => (
                <Toy toy={toy} key={`${toy.toy_name} ${toy.toy_id}`} />
              ))}
            </AnimatePresence>
          </div>
        )}
        <ScanButton />
      </div>
      <Footer />
    </>
  );
}
