import VrchatLogo from "../../assets/vrchat-192x192.png";
import DiscordLogo from "../../assets/discord-mark-white.svg";
import GithubLogo from "../../assets/GitHub-Mark-Light-64px.png";
import Switch from "../Switch";
import ExternalLogo from "./ExternalLogo";
import { useVersion } from "../../hooks/useVersion";
import { Cog6ToothIcon } from "@heroicons/react/20/solid";
import { useEffect, useState } from "react";
import { useCoreEventContext } from "../../context/CoreEventContext";
import SettingsModal from "../SettingsModal";
import UpdatePing from "./UpdatePing";
import { useUpdate } from "../../hooks/useUpdate";

export function Footer() {
  const [settingsIsOpen, setSettingsIsOpen] = useState(false);
  const { config, refreshConfig, isEnabled, toggleIsEnabled } =
    useCoreEventContext();
  const { canUpdate } = useUpdate();
  const { version } = useVersion();

  useEffect(() => {
    // Refresh from backend when modal is closed
    if (settingsIsOpen) return;
    refreshConfig();
  }, [settingsIsOpen]);

  return (
    <div className="grid grid-cols-3 items-center fixed left-0 bottom-4 min-w-full px-8">
      <div className="flex justify-around items-center">
        <UpdatePing canUpdate={canUpdate}>
          <Cog6ToothIcon
            className={`h-10 cursor-pointer transform duration-300 ease-in-out ${
              settingsIsOpen && "rotate-45"
            }`}
            onClick={() => {
              if (config != null) setSettingsIsOpen(true);
            }}
          />
        </UpdatePing>
        {config != null && (
          <SettingsModal
            isOpen={settingsIsOpen}
            onClose={() => setSettingsIsOpen(false)}
            config={config}
            canUpdate={canUpdate}
          />
        )}
        <Switch isEnabled={isEnabled} toggleIsEnabled={toggleIsEnabled} />
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
