import { invoke } from "@tauri-apps/api";
import type { FeSocialLink } from "../../src-tauri/bindings/FeSocialLink";
import { OPEN_BROWSER } from "../data/constants";
import Tooltip from "../layout/Tooltip";
import { createToast } from "./Toast";

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
  async function openBrowser() {
    try {
      await invoke(OPEN_BROWSER, { link: link });
    } catch (e) {
      createToast("error", "Could not open browser", JSON.stringify(e));
    }
  }
  return (
    <Tooltip text={tooltip}>
      <img className="h-6 cursor-pointer" src={src} onClick={openBrowser} />
    </Tooltip>
  );
}
