import lovenseLogo from "../../assets/Lovense.png";
import lovenseConnectLogo from "../../assets/Lovense_Connect.png";
import type { ReactNode } from "react";

export default function ({ name }: { name: string }) {
  let toyName = name;
  let badge: ReactNode = null;
  if (name.startsWith("Lovense Connect")) {
    toyName = name.replace("Lovense Connect ", "");
    badge = <img className="max-h-6 rounded-lg" src={lovenseConnectLogo} />;
  } else if (name.startsWith("Lovense")) {
    toyName = name.replace("Lovense ", "");
    badge = <img className="max-h-6 rounded-lg" src={lovenseLogo} />;
  }

  return (
    <div className="flex gap-x-4 items-center">
      {toyName}
      {badge}
    </div>
  );
}
