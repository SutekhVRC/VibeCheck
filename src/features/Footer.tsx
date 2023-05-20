import VrchatLogo from "../assets/vrchat-192x192.png";
import DiscordLogo from "../assets/discord-mark-white.svg";
import GithubLogo from "../assets/GitHub-Mark-Light-64px.png";
import Switch from "../layout/Switch";
import ExternalLogo from "../components/ExternalLogo";
import { useVersion } from "../hooks/useVersion";
import { Cog6ToothIcon } from "@heroicons/react/20/solid";
import { useEffect, useState } from "react";
import { useCoreEventContext } from "../context/CoreEventContext";
import Config from "./Config";
import UpdatePing from "../components/UpdatePing";
import { useUpdate } from "../hooks/useUpdate";

export function Footer() {
  const [configIsOpen, setConfigIsOpen] = useState(false);
  const { config, refreshConfig, isEnabled, toggleIsEnabled } =
    useCoreEventContext();
  const { canUpdate } = useUpdate();
  const { version } = useVersion();

  useEffect(() => {
    // Refresh from backend when modal is closed
    if (configIsOpen) return;
    refreshConfig();
  }, [configIsOpen]);

  return (
    <div className="grid grid-cols-3 items-center fixed left-0 bottom-4 min-w-full px-8">
      <div className="flex justify-around items-center">
        <UpdatePing canUpdate={canUpdate}>
          <Cog6ToothIcon
            className={`h-10 cursor-pointer transform duration-300 ease-in-out ${
              configIsOpen && "rotate-45"
            }`}
            onClick={() => {
              if (config != null) setConfigIsOpen(true);
            }}
          />
        </UpdatePing>
        {config != null && (
          <Config
            isOpen={configIsOpen}
            onClose={() => setConfigIsOpen(false)}
            config={config}
            canUpdate={canUpdate}
          />
        )}
        <Switch
          theme="red-green"
          isEnabled={isEnabled}
          toggleIsEnabled={toggleIsEnabled}
        />
      </div>
      {version}
      <div className="flex justify-around items-center">
        <ExternalLogo
          src={VrchatLogo}
          link="VRChatGroup"
          tooltip="Vibecheck VRChat Group"
        />
        <ExternalLogo
          src={DiscordLogo}
          link="Discord"
          tooltip="Vibecheck Discord"
        />
        <ExternalLogo
          src={GithubLogo}
          link="Github"
          tooltip="Vibecheck Github"
        />
      </div>
    </div>
  );
}
