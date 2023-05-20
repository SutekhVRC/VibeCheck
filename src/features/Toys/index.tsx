import { useCoreEventContext } from "../../context/CoreEventContext";
import { useToys } from "../../context/ToysContext";
import cryingAnimeGirl from "../../assets/menhera_chan.gif";
import ScanButton from "../../components/ScanButton";
import Toy from "./Toy";

export default function Toys() {
  const { toys } = useToys();
  const { isScanning, toggleScan } = useCoreEventContext();

  return (
    <div className="flex-col justify-between items-stretch text-lg rounded-lg p-4 bg-zinc-800 h-[600px]">
      {Object.keys(toys).length == 0 ? (
        <div className="flex justify-center items-center">
          No Toys
          <img src={cryingAnimeGirl} />
        </div>
      ) : (
        <div className="overflow-y-scroll pl-2 scrollbar pt-2 pb-2 max-h-[520px]">
          {Object.values(toys).map((toy) => (
            <Toy toy={toy} key={`${toy.toy_name} ${toy.toy_id}`} />
          ))}
        </div>
      )}
      <ScanButton isScanning={isScanning} toggleScan={toggleScan} />
    </div>
  );
}
