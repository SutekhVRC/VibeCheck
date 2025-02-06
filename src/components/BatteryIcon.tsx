import {
  BatteryFull,
  BatteryLow,
  BatteryMedium,
  BatteryWarning,
  Hourglass,
  Plug,
  WifiOff,
} from "lucide-react";
import type { CSSProperties } from "react";
import type { ToyPower } from "src-tauri/bindings/ToyPower";
import Tooltip from "../layout/Tooltip";
import { percent } from "../utils";

export default function Batteryicon({ toyPower }: { toyPower: ToyPower }) {
  let tooltipText = "";
  let iconComponent: JSX.Element;
  let iconStyle: CSSProperties = {};

  if (toyPower == "NoBattery") {
    tooltipText = "No battery";
    iconComponent = <Plug className="h-6" />;
  } else if (toyPower == "Offline") {
    tooltipText = "Offline";
    iconComponent = <WifiOff className="h-6" />;
  } else if (toyPower == "Pending") {
    tooltipText = "Pending";
    iconComponent = <Hourglass className="h-6" />;
  } else {
    const battery = toyPower.Battery;
    tooltipText = percent.format(battery);
    iconStyle = { color: `hsl(${battery * 120}, 75%, 50%)` };
    if (battery <= 0.1) {
      iconComponent = <BatteryWarning className="h-8" />;
    } else if (battery <= 0.4) {
      iconComponent = <BatteryLow className="h-8" />;
    } else if (battery <= 0.7) {
      iconComponent = <BatteryMedium className="h-8" />;
    } else {
      iconComponent = <BatteryFull className="h-8" />;
    }
  }

  return (
    <Tooltip text={tooltipText}>
      <span className="cursor-help" style={iconStyle}>
        {iconComponent}
      </span>
    </Tooltip>
  );
}
