import { ReactElement } from "react";
import Badge from "react-bootstrap/esm/Badge";
import lovenseLogo from "../assets/Lovense.png";
import lovenseConnectLogo from "../assets/Lovense_Connect.png";

export default function (props: { name: string }) {
  let badge: ReactElement;
  if (props.name.startsWith("Lovense Connect")) {
    badge = (
      <img
        src={lovenseConnectLogo}
        style={{ maxHeight: "1.5rem", borderRadius: ".5rem" }}
      />
    );
  } else if (props.name.startsWith("Lovense")) {
    badge = (
      <img
        src={lovenseLogo}
        style={{ maxHeight: "1.5rem", borderRadius: ".5rem" }}
      />
    );
  } else {
    badge = (
      <Badge pill bg="light" text="dark" style={{ fontSize: ".75rem" }}>
        Other
      </Badge>
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
      {props.name.replace("Lovense Connect ", "").replace("Lovense ", "")}
      {badge}
    </div>
  );
}
