import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import { OSC_DATA_PREFIX } from "../data/constants";
import Switch from "../layout/Switch";
import { ToyAnatomyArray } from "../data/stringArrayTypes";
import { FeVCToyAnatomy } from "../../src-tauri/bindings/FeVCToyAnatomy";
import FourPanel from "../components/FourPanel";
import { handleToyAlter } from "../hooks/useToys";
import { createToast } from "../components/Toast";
import { Select } from "../layout/Select";
import { ClipboardIcon } from "@heroicons/react/24/solid";

export default function ToySettings({ toy }: { toy: FeVCToy }) {
  const parsed_toy_name = toy.toy_name
    .replace("Lovense Connect", "Lovense")
    .replaceAll(" ", "_")
    .toLowerCase();
  const osc_data_addr = `${OSC_DATA_PREFIX}${parsed_toy_name}/${toy.sub_id}/battery`;

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(osc_data_addr);
      createToast("info", "Copied to clipboard", osc_data_addr);
    } catch (e) {
      createToast("error", "Could not copy to clipboard!", JSON.stringify(e));
    }
  }

  return (
    <div className="pb-4 text-sm">
      <div className="grid grid-cols-[minmax(4rem,_1fr)_1fr_minmax(4rem,_3fr)_minmax(2.5rem,_1fr)] text-sm text-justify p-4 gap-y-1 gap-x-2 md:gap-x-8">
        <div className="flex items-center gap-1">
          OSC Data
          {toy.toy_connected && (
            <ClipboardIcon
              onClick={handleCopy}
              className="h-4 cursor-pointer"
            />
          )}
        </div>
        <Switch
          size="small"
          checked={toy.osc_data}
          onChange={(e) => handleToyAlter({ ...toy, osc_data: e })}
        />
        <div></div>
        <div></div>
        <FourPanel
          text="Anatomy"
          three={
            <Select
              defaultValue={toy.toy_anatomy}
              onChange={(e) =>
                handleToyAlter({
                  ...toy,
                  toy_anatomy: e.target.value as FeVCToyAnatomy,
                })
              }
              options={ToyAnatomyArray}
            />
          }
        />
      </div>
    </div>
  );
}
