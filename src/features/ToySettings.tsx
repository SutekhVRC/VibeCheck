import type { FeVCToy } from "@bindings/FeVCToy";
import { FeVCToyAnatomy } from "@bindings/FeVCToyAnatomy";
import { ClipboardCopy } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import FourPanel from "../components/FourPanel";
import FourPanelContainer from "../components/FourPanelContainer";
import { OSC, TOOLTIP } from "../data/constants";
import { ToyAnatomyArray } from "../data/stringArrayTypes";
import { handleToyAlter } from "../hooks/useToys";
import { Select } from "../layout/Select";
import Slider from "../layout/Slider";
import Switch from "../layout/Switch";
import Tooltip, { TooltipLabel } from "../layout/Tooltip";

export default function ToySettings({ toy }: { toy: FeVCToy }) {
  const BT_UPDATE_DEFAULT = 20;
  const BT_UPDATE_MIN = 1;
  const BT_UPDATE_MAX = 500;

  function clampUpdateRate(rate: number) {
    return Math.min(BT_UPDATE_MAX, Math.max(BT_UPDATE_MIN, rate));
  }

  function parseUpdateRate(rate: bigint) {
    const numericRate = Number(rate);
    if (!Number.isFinite(numericRate)) return BT_UPDATE_DEFAULT;
    return clampUpdateRate(numericRate);
  }

  const [btUpdateRate, setBtUpdateRate] = useState(
    parseUpdateRate(toy.bt_update_rate),
  );

  useEffect(() => {
    setBtUpdateRate(parseUpdateRate(toy.bt_update_rate));
  }, [toy.bt_update_rate, toy.toy_id, toy.sub_id]);

  function handleBtUpdateRateCommit(value?: number) {
    const nextRate = clampUpdateRate(value ?? BT_UPDATE_DEFAULT);
    setBtUpdateRate(nextRate);
    handleToyAlter({ ...toy, bt_update_rate: BigInt(nextRate) });
  }

  const parsed_toy_name = toy.toy_name.replaceAll(" ", "_").toLowerCase();
  const osc_data_addr = `${OSC.DATA_PREFIX}${parsed_toy_name}/${toy.sub_id}/battery`;

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(osc_data_addr);
      toast.info(`Copied to clipboard\n${osc_data_addr}`);
    } catch (e) {
      toast.error(`Could not copy to clipboard!\n${JSON.stringify(e)}`);
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
        <FourPanel
          text="Toy Update Rate"
          tooltip={TOOLTIP.ToyUpdateRate}
          three={
            <Slider
              min={BT_UPDATE_MIN}
              max={BT_UPDATE_MAX}
              step={1}
              value={[btUpdateRate]}
              onValueChange={(value) =>
                setBtUpdateRate(clampUpdateRate(value[0] ?? BT_UPDATE_DEFAULT))
              }
              onValueCommit={(value) => handleBtUpdateRateCommit(value[0])}
            />
          }
          four={`${btUpdateRate}hz`}
        />
      </FourPanelContainer>
    </div>
  );
}
