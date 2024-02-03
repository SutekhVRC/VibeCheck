import { ObjectValues } from "@/utils";
import { invoke } from "@tauri-apps/api";
import { INVOKE, TOOLTIP } from "../data/constants";
import Tooltip from "../layout/Tooltip";
import { createToast } from "./Toast";

type ExternalLogoProps = {
  src: string;
  tooltip: ObjectValues<typeof TOOLTIP>;
};

export default function ExternalLogo({ src, tooltip }: ExternalLogoProps) {
  async function openBrowser() {
    try {
      await invoke(INVOKE.OPEN_BROWSER, { link: tooltip.link });
    } catch (e) {
      createToast("error", "Could not open browser", JSON.stringify(e));
    }
  }
  return (
    <Tooltip text={tooltip.text}>
      <img className="h-6 cursor-pointer" src={src} onClick={openBrowser} />
    </Tooltip>
  );
}
