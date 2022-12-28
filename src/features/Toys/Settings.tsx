import { invoke } from "@tauri-apps/api";
import { useState } from "react";
import type { FeVCToy } from "../../../src-tauri/bindings/FeVCToy";
import { ALTER_TOY } from "../../data/constants";
import Modal from "../../layout/Modal";
import Toast from "../../layout/Toast";

export default function ({
  onClose,
  toy,
}: {
  onClose: () => void;
  toy: FeVCToy;
}) {
  const [oscData, setOscData] = useState(toy.osc_data);

  const parsed_toy_name = toy.toy_name
    .replace("Lovense Connect", "Lovense")
    .replaceAll(" ", "_")
    .toLowerCase();
  const osc_data_addr = `${parsed_toy_name}/${toy.sub_id}/battery`;

  async function onSave(newOSCDataState: boolean) {
    await invoke(ALTER_TOY, {
      toyId: toy.toy_id,
      mutate: { OSCData: newOSCDataState },
    });
  }

  function handleOnClose() {
    if (oscData != toy.osc_data) {
      onSave(oscData);
    }
    onClose();
  }

  const copy = async () => {
    await navigator.clipboard.writeText(osc_data_addr);
  };

  return (
    <Modal onClose={handleOnClose} title={toy.toy_name}>
      <div className="grid grid-cols-2 gap-y-2 justify-items-end">
        <label className="justify-self-start">OSC Data</label>
        <input
          type="checkbox"
          checked={oscData}
          onChange={() => setOscData((e) => !e)}
        />
      </div>
      <div>
        <div className="flex justify-center">
          <Toast buttonText={osc_data_addr} toastText="Copied" onClick={copy} />
        </div>
      </div>
    </Modal>
  );
}
