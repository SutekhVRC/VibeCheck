import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import { OSC_DATA_PREFIX } from "../data/constants";
import Switch from "../layout/Switch";
import { useToastContext } from "../context/ToastContext";
import Button from "../layout/Button";
import { ToyAnatomyArray } from "../data/stringArrayTypes";
import { FeVCToyAnatomy } from "../../src-tauri/bindings/FeVCToyAnatomy";

export default function ToySettings({
  toy,
  handleToyAlter,
}: {
  toy: FeVCToy;
  handleToyAlter: (newToy: FeVCToy) => void;
}) {
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
      toast.createToast("Could not copy to clipboard!", `${e}`, "error");
    }
  }

  return (
    <div className="pb-4 text-sm">
      <div className="grid grid-cols-[2fr,_1fr,_6fr] text-sm text-justify gap-y-1 p-4 items-center">
        <label>OSC Data</label>
        <Switch
          size="small"
          isEnabled={toy.osc_data}
          toggleIsEnabled={(e) => handleToyAlter({ ...toy, osc_data: e })}
        />
        <div />
        <label>Anatomy</label>
        <div />
        {/* I don't think we can really use headless listbox, there's problems with relative/absolute position lifting the toy flexbox up */}
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
      </div>
      <Button onClick={handleCopy}>Copy osc data parameter</Button>
    </div>
  );
}
