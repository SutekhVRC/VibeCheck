import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import type { FeVCToy } from "../../../src-tauri/bindings/FeVCToy";
import { ALTER_TOY } from "../../data/constants";
import Toast from "../../layout/Toast";
import Switch from "../../layout/Switch";

export default function ToySettings({ toy }: { toy: FeVCToy }) {
  const [oscData, setOscData] = useState(toy.osc_data);

  const parsed_toy_name = toy.toy_name
    .replace("Lovense Connect", "Lovense")
    .replaceAll(" ", "_")
    .toLowerCase();
  const osc_data_addr = `${parsed_toy_name}/${toy.sub_id}/battery`;

  useEffect(() => {
    async function saveConfig(newOSCDataState: boolean) {
      try {
        await invoke(ALTER_TOY, {
          toyId: toy.toy_id,
          mutate: { OSCData: newOSCDataState },
        });
      } catch (e) {
        alert(e);
      }
    }
    saveConfig(oscData);
  }, [oscData]);

  const copy = async () => {
    await navigator.clipboard.writeText(osc_data_addr);
  };

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
      <Toast
        buttonText="Click to copy osc data address"
        title="Copied to clipboard"
        description={osc_data_addr}
        onClick={copy}
      />
    </div>
  );
}
