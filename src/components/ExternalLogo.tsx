import { ObjectValues } from "@/utils";
import { invoke } from "@tauri-apps/api";
import { toast } from "sonner";
import { INVOKE, TOOLTIP } from "../data/constants";
import Tooltip from "../layout/Tooltip";

type ExternalLogoProps = {
  src: string;
  tooltip: ObjectValues<typeof TOOLTIP>;
};

export default function ExternalLogo({ src, tooltip }: ExternalLogoProps) {
  async function openBrowser() {
    try {
      await invoke(INVOKE.OPEN_BROWSER, { link: tooltip.link });
    } catch (e) {
      toast.error(`Could not open browser\n${JSON.stringify(e)}`);
    }
  }
  return (
    <Tooltip text={tooltip.text}>
      <img className="h-6 cursor-pointer" src={src} onClick={openBrowser} />
    </Tooltip>
  );
}
