import { useState } from "react";
import { useCoreEventContext } from "../context/CoreEventContext";
import SettingsModal from "./SettingsModal";

export default function () {
  const [settingsIsOpen, setSettingsIsOpen] = useState(false);
  const { isEnabled, toggleIsEnabled } = useCoreEventContext();

  return (
    <div>
      <button
        className="text-6xl font-bold p-2 w-1/2 border-gray-500 border-solid border-2 rounded-md hover:border-gray-300"
        onClick={() => setSettingsIsOpen(true)}
      >
        Settings
      </button>
      <SettingsModal
        open={settingsIsOpen}
        onClose={() => setSettingsIsOpen(false)}
      />
      <button
        className={`text-6xl font-bold p-2 w-1/2 border-gray-500 border-solid border-2 rounded-md hover:border-gray-300
          ${isEnabled ? " grad-forewards" : ""}`}
        onClick={() => toggleIsEnabled()}
      >
        {isEnabled ? "Disable" : "Enable"}
      </button>
    </div>
  );
}
