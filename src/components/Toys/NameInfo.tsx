import lovenseLogo from "../../assets/Lovense.png";
import lovenseConnectLogo from "../../assets/Lovense_Connect.png";
import type { ReactNode } from "react";
import Tooltip from "../Tooltip";
import {
  Battery100Icon,
  Battery50Icon,
  Battery0Icon,
} from "@heroicons/react/20/solid";

export default function ({ name, battery }: { name: string; battery: number }) {
  let toyName = name;
  let badge: ReactNode = null;
  if (name.startsWith("Lovense Connect")) {
    toyName = name.replace("Lovense Connect ", "");
    badge = (
      <Tooltip text="Lovense Connect">
        <img className="max-h-6 rounded-lg" src={lovenseConnectLogo} />;
      </Tooltip>
    );
  } else if (name.startsWith("Lovense")) {
    toyName = name.replace("Lovense ", "");
    badge = (
      <Tooltip text="Lovense">
        <img className="max-h-6 rounded-lg" src={lovenseLogo} />
      </Tooltip>
    );
  }

  return (
    <div className="flex gap-x-4 items-center">
      {toyName}
      {badge}
      <Tooltip text={`${battery * 100}%`}>
        <div
          style={{
            color: `hsl(${battery * 120}, 75%, 50%)`,
          }}
        >
          {battery <= 0.15 ? (
            <Battery0Icon className="h-8" />
          ) : battery <= 0.5 ? (
            <Battery50Icon className="h-8" />
          ) : (
            <Battery100Icon className="h-8" />
          )}
        </div>
      </Tooltip>
    </div>
  );
}
