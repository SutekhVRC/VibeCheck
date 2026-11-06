import Tooltip from "@/layout/Tooltip";
import { mix } from "@/theme/color";
import { useTheme } from "@/theme/ThemeProvider";
import { percent } from "@/utils";
import { ToyPower } from "@bindings/ToyPower";
import {
  BatteryFull,
  BatteryLow,
  BatteryMedium,
  BatteryWarning,
  Hourglass,
  Plug,
  WifiOff,
} from "lucide-react";
import { CSSProperties } from "react";

export default function Batteryicon({ toyPower }: { toyPower: ToyPower }) {
  const { colors } = useTheme().theme;
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
    // Theme danger colour when empty -> success colour when full
    iconStyle = { color: mix(colors.danger, colors.success, battery) };
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
      <span className="cursor-help text-theme-muted" style={iconStyle}>
        {iconComponent}
      </span>
    </Tooltip>
  );
}
