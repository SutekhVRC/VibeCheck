import { ObjectValues } from "@/utils";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { INVOKE, TOOLTIP } from "../data/constants";
import Tooltip from "../layout/Tooltip";

type ExternalLogoProps = {
  src: string;
  tooltip: ObjectValues<typeof TOOLTIP>;
  className?: string;
};

export default function ExternalLogo({
  src,
  tooltip,
  className = "h-6",
}: ExternalLogoProps) {
  async function openBrowser() {
    try {
      await invoke(INVOKE.OPEN_BROWSER, { link: tooltip.link });
    } catch (e) {
      toast.error(`Could not open browser\n${JSON.stringify(e)}`);
    }
  }
  return (
    <Tooltip text={tooltip.text}>
      <img
        className={`${className} cursor-pointer opacity-80 transition-opacity hover:opacity-100`}
        src={src}
        onClick={openBrowser}
      />
    </Tooltip>
  );
}
