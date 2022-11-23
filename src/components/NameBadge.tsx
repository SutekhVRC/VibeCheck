import { ReactNode } from "react";
import lovenseLogo from "../assets/Lovense.png";
import lovenseConnectLogo from "../assets/Lovense_Connect.png";
import { Tooltip, OverlayTrigger } from "react-bootstrap";

export default function ({ name }: { name: string }) {
  let badge: ReactNode = null;
  if (name.startsWith("Lovense Connect")) {
    badge = (
      <OverlayTrigger overlay={<Tooltip>Lovense Connect</Tooltip>}>
        <img
          src={lovenseConnectLogo}
          style={{ maxHeight: "1.5rem", borderRadius: ".5rem" }}
        />
      </OverlayTrigger>
    );
  } else if (name.startsWith("Lovense")) {
    badge = (
      <OverlayTrigger overlay={<Tooltip>Lovense BTLE</Tooltip>}>
        <img
          src={lovenseLogo}
          style={{ maxHeight: "1.5rem", borderRadius: ".5rem" }}
        />
      </OverlayTrigger>
    );
  }

  return (
    <div
      style={{
        display: "flex",
        columnGap: "1rem",
        alignItems: "center",
      }}
    >
      {name.replace("Lovense Connect ", "").replace("Lovense ", "")}
      {badge}
    </div>
  );
}
