import { SettingRow } from "@/components/SettingRow";
import type { FeVCToy } from "@bindings/FeVCToy";
import { ClipboardCopy } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import EditableNumber from "../components/EditableNumber";
import { OSC, TOOLTIP } from "../data/constants";
import { ToyAnatomyArray } from "../data/stringArrayTypes";
import { handleToyAlter } from "../hooks/useToys";
import { Select } from "../layout/Select";
import Slider from "../layout/Slider";
import Switch from "../layout/Switch";
import Tooltip from "../layout/Tooltip";

/** Toy-wide settings, shown in the strip under the toy's name */
export default function ToySettings({ toy }: { toy: FeVCToy }) {
  const BT_UPDATE_DEFAULT = 20;
  const BT_UPDATE_MIN = 1;
  const BT_UPDATE_MAX = 20;

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
    <div className="grid grid-cols-1 gap-x-6 rounded-2xl border border-theme-border/60 bg-theme-raised px-4 py-1 text-left text-sm md:grid-cols-3">
      <SettingRow
        label="OSC Data"
        tooltip={TOOLTIP.OSC_Data}
        extra={
          toy.toy_connected && (
            <Tooltip text="Copy osc data address to clipboard">
              <button
                type="button"
                aria-label="Copy osc data address"
                onClick={handleCopy}
                className="text-theme-secondary hover:text-theme-primary"
              >
                <ClipboardCopy className="h-4 w-4" />
              </button>
            </Tooltip>
          )
        }
        control={
          <Switch
            size="small"
            checked={toy.osc_data}
            onCheckedChange={(checked) =>
              handleToyAlter({ ...toy, osc_data: checked })
            }
          />
        }
      />
      <SettingRow label="Anatomy" tooltip={TOOLTIP.Anatomy}>
        <Select
          value={toy.toy_anatomy}
          onValueChange={(anatomy) =>
            handleToyAlter({ ...toy, toy_anatomy: anatomy })
          }
          options={ToyAnatomyArray}
        />
      </SettingRow>
      <SettingRow
        label="Update Rate"
        tooltip={TOOLTIP.ToyUpdateRate}
        control={
          <EditableNumber
            value={btUpdateRate}
            min={BT_UPDATE_MIN}
            max={BT_UPDATE_MAX}
            step={1}
            suffix="hz"
            onCommit={handleBtUpdateRateCommit}
            label="Toy update rate"
          />
        }
      >
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
      </SettingRow>
    </div>
  );
}
