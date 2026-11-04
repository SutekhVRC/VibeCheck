import EditableNumber from "@/components/EditableNumber";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { TOOLTIP } from "@/data/constants";
import useSimulate from "@/hooks/useSimulate";
import Button from "@/layout/Button";
import Slider from "@/layout/Slider";
import Switch from "@/layout/Switch";
import Tooltip from "@/layout/Tooltip";
import type { FeVCToy } from "@bindings/FeVCToy";
import type { FeVCToyFeature } from "@bindings/FeVCToyFeature";
import { Play } from "lucide-react";

const triggerClass = "m-0 flex items-center gap-1.5 px-3 py-0.5 text-sm";

/*
 * "Test" button for the feature header. Opens a small panel to run the
 * feature at a chosen power. The panel uses the existing simulate logic,
 * which stops the motor when the panel closes (it unmounts).
 */
export default function FeatureTest({
  toy,
  feature,
  label,
}: {
  toy: FeVCToy;
  feature: FeVCToyFeature;
  label: string;
}) {
  const unavailableReason =
    toy.toy_id == null
      ? "Connect the toy to test it"
      : feature.feature_levels.idle_level > 0
        ? "Set Idle Speed to 0 to test this feature"
        : null;

  if (unavailableReason) {
    return (
      <Tooltip text={unavailableReason}>
        {/* span: disabled buttons don't send the hover events tooltips need */}
        <span>
          <Button variant="secondary" disabled className={triggerClass}>
            <Play className="h-3.5 w-3.5" />
            Test
          </Button>
        </span>
      </Tooltip>
    );
  }

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button variant="secondary" className={triggerClass}>
          <Play className="h-3.5 w-3.5" />
          Test
        </Button>
      </PopoverTrigger>
      <PopoverContent
        align="end"
        sideOffset={8}
        className="w-80 rounded-2xl border-theme-border text-sm shadow-theme-pop"
      >
        <TestPanel toy={toy} feature={feature} label={label} />
      </PopoverContent>
    </Popover>
  );
}

function TestPanel({
  toy,
  feature,
  label,
}: {
  toy: FeVCToy;
  feature: FeVCToyFeature;
  label: string;
}) {
  const {
    simulateEnabled,
    simulateLevel,
    toggleSimulate,
    simulateOnValueChange,
    simulateOnValueCommit,
  } = useSimulate(toy.toy_id, feature);

  if (simulateEnabled === null) {
    return (
      <div className="text-theme-muted">This feature can&apos;t be tested.</div>
    );
  }

  return (
    <div className="flex flex-col gap-3 text-left">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="font-semibold text-theme-text">Test {label}</div>
          <div className="text-xs text-theme-muted">
            {TOOLTIP.Simulate.text}
          </div>
        </div>
        <Switch
          checked={simulateEnabled}
          onCheckedChange={toggleSimulate}
          aria-label="Run test"
        />
      </div>
      <div className="flex items-center gap-3">
        <div className="min-w-0 flex-1">
          <Slider
            multiply={100}
            disabled={!simulateEnabled}
            min={0}
            max={1}
            step={0.01}
            value={[simulateLevel]}
            onValueChange={(e) => simulateOnValueChange(e[0])}
            onValueCommit={() => simulateOnValueCommit()}
          />
        </div>
        <EditableNumber
          value={simulateLevel}
          min={0}
          max={1}
          step={0.01}
          scale={100}
          disabled={!simulateEnabled}
          onCommit={(v) => simulateOnValueChange(v)}
          label="Test power"
        />
      </div>
      <div className="text-xs text-theme-muted">
        Stops when you close this panel.
      </div>
    </div>
  );
}
