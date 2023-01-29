import { invoke } from "@tauri-apps/api";
import type { FeSocialLink } from "../../src-tauri/bindings/FeSocialLink";
import { OPEN_BROWSER } from "../data/constants";
import Tooltip from "../layout/Tooltip";

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
    await invoke(OPEN_BROWSER, { link: link });
  }
  return (
    <Tooltip text={tooltip}>
      <img className="max-h-8 cursor-pointer" src={src} onClick={openBrowser} />
    </Tooltip>
  );
}
