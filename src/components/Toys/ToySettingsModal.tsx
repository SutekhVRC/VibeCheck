import { invoke } from "@tauri-apps/api";
import { useState } from "react";
import type { FeVCToy } from "../../../src-tauri/bindings/FeVCToy";
import { ALTER_TOY } from "../../data/constants";
import Modal from "../Modal";

export default function ({
  isOpen,
  onClose,
  toy,
}: {
  isOpen: boolean;
  onClose: () => void;
  toy: FeVCToy;
}) {
  const [oscData, setOscData] = useState(toy.osc_data);

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

  return (
    <Modal isOpen={isOpen} onClose={handleOnClose} title={toy.toy_name}>
      <div className="grid grid-cols-2 gap-y-2 pb-4 justify-items-end">
        <label className="justify-self-start">OSC Data</label>
        <input
          type="checkbox"
          checked={oscData}
          onChange={() => setOscData((e) => !e)}
        />
      </div>
    </Modal>
  );
}
