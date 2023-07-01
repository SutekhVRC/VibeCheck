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
import FourPanelContainer from "../components/FourPanelContainer";
import Tooltip, { TooltipLabel } from "../layout/Tooltip";

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
      <FourPanelContainer>
        <div className="flex items-center gap-1">
          <TooltipLabel
            text="OSC Data"
            tooltip="If vibecheck should use OSC data"
          />
          {toy.toy_connected && (
            <Tooltip text="Copy osc data address to clipboard">
              <ClipboardIcon
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
          tooltip="Anatomy types can be used as a category filter to turn on/off multiple toys at the same time"
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
