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
    <>
      <div className="flex flex-col gap-4">
        <img className="h-20 object-contain -mt-6" src={VibecheckLogo} />
        <div className="flex flex-col justify-between gap-2 text-lg rounded-lg p-3 bg-zinc-800 h-[620px]">
          {Object.keys(toys).length == 0 ? (
            <div className="flex justify-center items-center">
              No Toys
              <img src={cryingAnimeGirl} />
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
      <Footer />
    </>
  );
}
