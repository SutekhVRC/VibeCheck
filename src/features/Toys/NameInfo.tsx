import lovenseLogo from "../../assets/Lovense.png";
import lovenseConnectLogo from "../../assets/Lovense_Connect.png";
import Tooltip from "../../layout/Tooltip";
import BatteryIcon from "../../components/BatteryIcon";

export default function NameInfo({
  name,
  battery,
}: {
  name: string;
  battery: number;
}) {
  return (
    <div className="flex gap-x-4 items-center">
      {name.startsWith("Lovense Connect ") ? (
        <>
          {name.replace("Lovense Connect ", "")}
          <Tooltip text="Lovense Connect">
            <img className="max-h-6 rounded-lg" src={lovenseConnectLogo} />
          </Tooltip>
        </>
      ) : name.startsWith("Lovense ") ? (
        <>
          {name.replace("Lovense ", "")}
          <Tooltip text="Lovense">
            <img className="max-h-6 rounded-lg" src={lovenseLogo} />
          </Tooltip>
        </>
      ) : (
        <>{name}</>
      )}
      <BatteryIcon battery={battery} />
    </div>
  );
}
