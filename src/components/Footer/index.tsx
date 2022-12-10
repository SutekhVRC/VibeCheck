import DiscordLogo from "../../assets/discord-mark-white.svg";
import GithubLogo from "../../assets/GitHub-Mark-Light-64px.png";
import ExternalLogo from "./ExternalLogo";
import Version from "./Version";

export function Footer() {
  return (
    <div className="grid grid-cols-2 fixed left-0 bottom-4 min-w-full">
      <Version />
      <div className="flex justify-evenly">
        <ExternalLogo src={DiscordLogo} link="https://discord.gg/g6kUFtMtpw" />
        <ExternalLogo
          src={GithubLogo}
          link="https://github.com/SutekhVRC/VibeCheck"
        />
      </div>
    </div>
  );
}
