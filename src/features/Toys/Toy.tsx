import { FeVCToy } from "../../../src-tauri/bindings/FeVCToy";

import lovenseLogo from "../../assets/Lovense.png";
import lovenseConnectLogo from "../../assets/Lovense_Connect.png";
import FeatureForm from "./FeatureForm";
import ToySettings from "./ToySettings";
import { FeatureDisclosure } from "./FeatureDisclosure";
import Tooltip from "../../layout/Tooltip";
import BatteryIcon from "../../components/BatteryIcon";

export default function Toy({ toy }: { toy: FeVCToy }) {
  const nameInfo = NameInfo(toy.toy_name);

  return (
    <div className="rounded-md bg-zinc-700 px-2 py-4 m-2">
      <div className="text-4xl flex justify-between items-center px-6">
        <div>{nameInfo.shortName}</div>
        <ToyInfo nameInfo={nameInfo} battery={toy.battery_level} />
      </div>
      <div className="grid m-2">
        <FeatureDisclosure title="Config">
          <ToySettings toy={toy} />
        </FeatureDisclosure>
        {toy.features.map((feature) => (
          <div
            className="flex flex-col"
            key={`${toy.toy_id} ${feature.feature_type} ${feature.feature_index}`}
          >
            <FeatureDisclosure
              title={`${feature.feature_type} ${feature.feature_index}`}
              titleIsOn={feature.feature_enabled}
            >
              <FeatureForm toyId={toy.toy_id} toyFeature={feature} />
            </FeatureDisclosure>
          </div>
        ))}
      </div>
    </div>
  );
}

type NameInfo = {
  shortName: string;
  fullName: string;
  logo: string | undefined;
};

function NameInfo(name: string): NameInfo {
  if (name.startsWith("Lovense Connect")) {
    return {
      fullName: name,
      shortName: name.replace("Lovense Connect ", ""),
      logo: lovenseConnectLogo,
    };
  } else if (name.startsWith("Lovense")) {
    return {
      fullName: name,
      shortName: name.replace("Lovense ", ""),
      logo: lovenseLogo,
    };
  }
  return {
    shortName: name,
    fullName: name,
    logo: undefined,
  };
}

function ToyInfo({
  nameInfo,
  battery,
}: {
  nameInfo: NameInfo;
  battery: number;
}) {
  return (
    <div className="flex gap-x-4 items-center">
      {nameInfo.logo != undefined && (
        <Tooltip text={nameInfo.fullName}>
          <img className="max-h-6 rounded-lg" src={nameInfo.logo} />
        </Tooltip>
      )}
      <BatteryIcon battery={battery} />
    </div>
  );
}
