import logo from "../assets/logo.png";
import githubLogo from "../assets/GitHub-Mark-Light-64px.png";
import discordLogo from "../assets/discord-mark-white.svg";
import { invoke } from "@tauri-apps/api";
import { VERSION } from "../data/constants";
import { useEffect, useState } from "react";

export default function () {
  const [version, setVersion] = useState("");

  useEffect(() => {
    async function getVersion() {
      await invoke<string>(VERSION).then((r) => {
        setVersion(r.replaceAll("-", " "));
      });
    }
    getVersion();
  }, []);

  return (
    <div className="header">
      <div className="grad-container">
        <img className="logo" src={logo} />
      </div>
      {version}
      <img
        className="extern-logo"
        src={discordLogo}
        onClick={() => open("https://discord.gg/g6kUFtMtpw")}
      />
      <img
        className="extern-logo"
        src={githubLogo}
        onClick={() => open("https://github.com/SutekhVRC/VibeCheck")}
      />
    </div>
  );
}
