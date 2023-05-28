import { invoke } from "@tauri-apps/api";
import type { FeSocialLink } from "../../src-tauri/bindings/FeSocialLink";
import { OPEN_BROWSER } from "../data/constants";
import Tooltip from "../layout/Tooltip";
import { useToastContext } from "../context/ToastContext";

type ExternalLogoProps = {
  src: string;
  link: FeSocialLink;
  tooltip: string;
};

export default function ExternalLogo({
  src,
  link,
  tooltip,
}: ExternalLogoProps) {
  const toast = useToastContext();
  async function openBrowser() {
    try {
      await invoke(OPEN_BROWSER, { link: link });
    } catch (e) {
      toast.createToast("Could not open browser", JSON.stringify(e), "error");
    }
  }
  return (
    <Tooltip text={tooltip}>
      <img className="max-h-8 cursor-pointer" src={src} onClick={openBrowser} />
    </Tooltip>
  );
}
