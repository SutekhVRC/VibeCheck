import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import { ALTER_TOY, OSC_DATA_PREFIX } from "../data/constants";
import Switch from "../layout/Switch";
import { useToastContext } from "../context/ToastContext";

export default function ToySettings({ toy }: { toy: FeVCToy }) {
  const [oscData, setOscData] = useState(toy.osc_data);

  const toast = useToastContext();

  const parsed_toy_name = toy.toy_name
    .replace("Lovense Connect", "Lovense")
    .replaceAll(" ", "_")
    .toLowerCase();
  const osc_data_addr = `${OSC_DATA_PREFIX}${parsed_toy_name}/${toy.sub_id}/battery`;

  useEffect(() => {
    async function saveConfig(newOSCDataState: boolean) {
      try {
        await invoke(ALTER_TOY, {
          toyId: toy.toy_id,
          mutate: { OSCData: newOSCDataState },
        });
      } catch (e) {
        toast.createToast("Could not alter toy!", `${e}`, "error");
      }
    }
    saveConfig(oscData);
  }, [oscData]);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(osc_data_addr);
      toast.createToast("Copied to clipboard", osc_data_addr, "info");
    } catch (e) {
      toast.createToast("Could not copy to clipboard!", `${e}`, "error");
    }
  }

  return (
    <div className="pb-4">
      <div className="grid grid-cols-[2fr,_1fr,_6fr] text-sm text-justify gap-y-1 p-4">
        <label>OSC Data</label>
        <Switch
          size="small"
          isEnabled={oscData}
          toggleIsEnabled={(e: boolean) => setOscData(e)}
        />
        <div />
        <div />
        <div />
      </div>
      <button
        onClick={handleCopy}
        className="border-2 px-2 rounded-sm border-zinc-500 text-sm"
      >
        Click to copy osc data parameter
      </button>
    </div>
  );
}
