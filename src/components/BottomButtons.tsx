import { useEffect, useState } from "react";
import { useCoreEventContext } from "../context/CoreEventContext";
import SettingsModal from "./SettingsModal";

export default function () {
  const [settingsIsOpen, setSettingsIsOpen] = useState(false);
  const { isEnabled, toggleIsEnabled, config, refreshConfig } =
    useCoreEventContext();

  useEffect(() => {
    // Refresh from backend when modal is closed
    if (settingsIsOpen) return;
    refreshConfig();
  }, [settingsIsOpen]);

  return (
    <div>
      <button
        className="text-6xl font-bold p-2 w-1/2 border-gray-500 border-solid border-2 rounded-md hover:border-gray-300"
        onClick={() => {
          if (config != null) setSettingsIsOpen(true);
        }}
      >
        Settings
      </button>
      {config != null && (
        <SettingsModal
          isOpen={settingsIsOpen}
          onClose={() => setSettingsIsOpen(false)}
          config={config}
        />
      )}
      <button
        className={`text-6xl font-bold p-2 w-1/2 border-gray-500 border-solid border-2 rounded-md hover:border-gray-300
          ${isEnabled && " grad-forewards"}`}
        onClick={() => toggleIsEnabled()}
      >
        {isEnabled ? "Disable" : "Enable"}
      </button>
    </div>
  );
}
