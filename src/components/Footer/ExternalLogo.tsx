import { invoke } from "@tauri-apps/api";
import type { FeSocialLink } from "../../../src-tauri/bindings/FeSocialLink";
import { OPEN_BROWSER } from "../../data/constants";

export default function ({ src, link }: { src: string; link: FeSocialLink }) {
  async function openBrowser() {
    await invoke(OPEN_BROWSER, { link: link });
  }
  return (
    <img className="max-h-8 cursor-pointer" src={src} onClick={openBrowser} />
  );
}
