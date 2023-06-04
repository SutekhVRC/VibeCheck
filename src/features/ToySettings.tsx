import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import { OSC_DATA_PREFIX } from "../data/constants";
import Switch from "../layout/Switch";
import { useToastContext } from "../context/ToastContext";
import Button from "../layout/Button";
import { ToyAnatomyArray } from "../data/stringArrayTypes";
import { FeVCToyAnatomy } from "../../src-tauri/bindings/FeVCToyAnatomy";
import FourPanel from "../components/FourPanel";
import { useToys } from "../hooks/useToys";

export default function ToySettings({ toy }: { toy: FeVCToy }) {
  const { handleToyAlter } = useToys();
  const toast = useToastContext();

  const parsed_toy_name = toy.toy_name
    .replace("Lovense Connect", "Lovense")
    .replaceAll(" ", "_")
    .toLowerCase();
  const osc_data_addr = `${OSC_DATA_PREFIX}${parsed_toy_name}/${toy.sub_id}/battery`;

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(osc_data_addr);
      toast.createToast("Copied to clipboard", osc_data_addr, "info");
    } catch (e) {
      toast.createToast(
        "Could not copy to clipboard!",
        JSON.stringify(e),
        "error"
      );
    }
  }

  return (
    <div className="pb-4 text-sm">
      <div className="grid grid-cols-[minmax(6rem,_1fr)_1fr_minmax(6rem,_3fr)_1fr] text-sm text-justify gap-y-1 p-4">
        <FourPanel
          text="OSC Data"
          two={
            <Switch
              size="small"
              checked={toy.osc_data}
              onChange={(e) => handleToyAlter({ ...toy, osc_data: e })}
            />
          }
        />
        <FourPanel
          text="Anatomy"
          three={
            <select
              className="outline-none text-zinc-800 px-2 rounded-sm"
              defaultValue={toy.toy_anatomy}
              onChange={(e) =>
                handleToyAlter({
                  ...toy,
                  toy_anatomy: e.target.value as FeVCToyAnatomy,
                })
              }
            >
              {ToyAnatomyArray.map((a) => (
                <option value={a} key={a}>
                  {a}
                </option>
              ))}
            </select>
          }
        />
        {/* I don't think we can really use headless listbox, there's problems with relative/absolute position lifting the toy flexbox up */}
      </div>
      {toy.toy_connected && (
        <Button onClick={handleCopy}>Copy osc data parameter</Button>
      )}
    </div>
  );
}
