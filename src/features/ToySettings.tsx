import { ClipboardCopy } from "lucide-react";
import type { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import { FeVCToyAnatomy } from "../../src-tauri/bindings/FeVCToyAnatomy";
import FourPanel from "../components/FourPanel";
import FourPanelContainer from "../components/FourPanelContainer";
import { createToast } from "../components/Toast";
import { OSC, TOOLTIP } from "../data/constants";
import { ToyAnatomyArray } from "../data/stringArrayTypes";
import { handleToyAlter } from "../hooks/useToys";
import { Select } from "../layout/Select";
import Switch from "../layout/Switch";
import Tooltip, { TooltipLabel } from "../layout/Tooltip";

export default function ToySettings({ toy }: { toy: FeVCToy }) {
  const parsed_toy_name = toy.toy_name
    .replace("Lovense Connect", "Lovense")
    .replaceAll(" ", "_")
    .toLowerCase();
  const osc_data_addr = `${OSC.DATA_PREFIX}${parsed_toy_name}/${toy.sub_id}/battery`;

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(osc_data_addr);
      createToast("info", "Copied to clipboard", osc_data_addr);
    } catch (e) {
      createToast("error", "Could not copy to clipboard!", JSON.stringify(e));
    }
  }

  return (
    <div className="rounded-md bg-zinc-700 px-4 text-sm">
      <FourPanelContainer>
        <div className="flex items-center gap-1">
          <TooltipLabel text="OSC Data" tooltip={TOOLTIP.OSC_Data} />
          {toy.toy_connected && (
            <Tooltip text="Copy osc data address to clipboard">
              <ClipboardCopy
                onClick={handleCopy}
                className="h-4 cursor-pointer"
              />
            </Tooltip>
          )}
        </div>
        <Switch
          size="small"
          checked={toy.osc_data}
          onCheckedChange={(checked) =>
            handleToyAlter({ ...toy, osc_data: checked })
          }
        />
        <div></div>
        <div></div>
        <FourPanel
          text="Anatomy"
          tooltip={TOOLTIP.Anatomy}
          three={
            <Select
              value={toy.toy_anatomy}
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
      </FourPanelContainer>
    </div>
  );
}
