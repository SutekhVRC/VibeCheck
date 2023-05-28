import {
  Battery0Icon,
  Battery100Icon,
  Battery50Icon,
  BoltSlashIcon,
} from "@heroicons/react/24/solid";
import Tooltip from "../layout/Tooltip";
import { percent } from "../utils";
import { CSSProperties } from "react";

export default function Batteryicon({ battery }: { battery: number | null }) {
  let tooltipText = "";
  let iconComponent: JSX.Element;
  let iconStyle: CSSProperties = {};

  if (battery == null) {
    tooltipText = "No battery";
    iconComponent = <BoltSlashIcon className="h-6" />;
  } else {
    tooltipText = percent.format(battery);
    iconStyle = { color: `hsl(${battery * 120}, 75%, 50%)` };
    if (battery <= 0.15) {
      iconComponent = <Battery0Icon className="h-8" />;
    } else if (battery <= 0.5) {
      iconComponent = <Battery50Icon className="h-8" />;
    } else {
      iconComponent = <Battery100Icon className="h-8" />;
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
