import { useAvatarCatalog } from "@/context/AvatarCatalog";
import Tooltip from "@/layout/Tooltip";
import { RefreshCw } from "lucide-react";

/**
 * A quiet line saying whether VibeCheck can see the current avatar's
 * parameters (for the picker and warnings), with a refresh button.
 */
export default function AvatarStatus() {
  const { catalog, refresh } = useAvatarCatalog();

  const text =
    catalog.status === "Ready"
      ? `Your avatar: ${catalog.parameters.length} ${
          catalog.parameters.length === 1 ? "parameter" : "parameters"
        }`
      : catalog.status === "Reading"
        ? "Reading your avatar from VRChat..."
        : catalog.status === "Looking"
          ? "Looking for VRChat..."
          : "Avatar parameters unavailable: VRChat not found";

  const tooltip =
    catalog.status === "Searching"
      ? "VibeCheck reads your avatar's parameters from VRChat to help with setup. Start VRChat with OSC on (Action Menu > Options > OSC)."
      : "Read from VRChat over OSCQuery. VibeCheck notices avatar changes on its own; refresh if something looks out of date.";

  return (
    <div className="flex min-w-0 items-center gap-1.5 text-xs font-normal normal-case tracking-normal text-theme-muted">
      <Tooltip text={tooltip}>
        <span className="truncate">{text}</span>
      </Tooltip>
      <Tooltip text="Read the avatar again">
        <button
          type="button"
          aria-label="Read the avatar again"
          onClick={refresh}
          className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-theme-raised hover:text-theme-text"
        >
          <RefreshCw
            className={
              catalog.status === "Reading" || catalog.status === "Looking"
                ? "h-3 w-3 animate-spin"
                : "h-3 w-3"
            }
          />
        </button>
      </Tooltip>
    </div>
  );
}
