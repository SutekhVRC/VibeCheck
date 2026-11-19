import { useCoreEventContext } from "@/context/CoreEvents";
import { INVOKE, TOOLTIP } from "@/data/constants";
import { Select } from "@/layout/Select";
import { TooltipLabel } from "@/layout/Tooltip";
import { errorText } from "@/lib/errors";
import { cn } from "@/lib/utils";
import type { FeToyConnectionState } from "@bindings/FeToyConnectionState";
import type { ToyConnection as Connection } from "@bindings/ToyConnection";
import { invoke } from "@tauri-apps/api/core";
import { KeyboardEvent, useEffect, useState } from "react";
import { toast } from "sonner";

/*
 * Settings > Toys > Toy connection: this computer's Bluetooth (VibeCheck's
 * own toy service), or Intiface Central on this computer or a phone, for
 * computers without Bluetooth (src-tauri util/toy_service.rs). Switching
 * applies at once: toys on the old connection disconnect.
 */

const THIS_COMPUTER = "This computer (Bluetooth)";
const INTIFACE = "Intiface Central";
const OPTIONS = [THIS_COMPUTER, INTIFACE] as const;

/** How often the status is refreshed while Settings is open */
const STATUS_EVERY_MS = 2000;

export default function ToyConnection() {
  const [state, setState] = useState<FeToyConnectionState | null>(null);
  // Chosen but not saved yet: Intiface without an address
  const [choosingIntiface, setChoosingIntiface] = useState(false);
  const [address, setAddress] = useState("");

  // The app's settings: reloaded after a reset, which also resets this
  const { config } = useCoreEventContext();

  useEffect(() => {
    let alive = true;
    async function refresh() {
      try {
        const next = await invoke<FeToyConnectionState>(
          INVOKE.GET_TOY_CONNECTION,
        );
        if (alive) setState(next);
      } catch {
        // Shown as unknown until the next try
      }
    }
    refresh();
    const timer = setInterval(refresh, STATUS_EVERY_MS);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [config]);

  // The saved address, when there is one. Only a change to it updates the
  // field: the status refresh every 2 s mustn't overwrite what's typed
  const saved = state?.connection;
  const savedAddress = saved?.mode === "Intiface" ? saved.address : null;
  useEffect(() => {
    if (savedAddress != null) setAddress(savedAddress);
  }, [savedAddress]);

  async function save(connection: Connection) {
    try {
      const done = await invoke<Connection>(INVOKE.SET_TOY_CONNECTION, {
        connection,
      });
      setChoosingIntiface(false);
      setState((s) =>
        s ? { connection: done, status: { state: "Connecting" } } : s,
      );
    } catch (e) {
      toast.error(errorText(e));
    }
  }

  if (!state) return null;
  const intiface = state.connection.mode === "Intiface" || choosingIntiface;
  const status = state.status;

  function onKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") e.currentTarget.blur();
  }

  function saveAddress() {
    const trimmed = address.trim();
    if (!trimmed) return;
    if (saved?.mode === "Intiface" && saved.address === trimmed) return;
    save({ mode: "Intiface", address: trimmed });
  }

  return (
    <div className="mb-3 flex flex-col gap-2">
      <div className="grid grid-cols-[minmax(8rem,1fr)_minmax(8rem,2fr)] items-center gap-x-4">
        <TooltipLabel text="Toy connection" tooltip={TOOLTIP.ToyConnection} />
        <Select
          name="Toy connection"
          value={intiface ? INTIFACE : THIS_COMPUTER}
          options={OPTIONS}
          onValueChange={(choice) => {
            if (choice === THIS_COMPUTER) {
              setChoosingIntiface(false);
              if (state.connection.mode !== "Local") save({ mode: "Local" });
            } else if (state.connection.mode !== "Intiface") {
              // Saved once there's an address
              setChoosingIntiface(true);
            }
          }}
        />
      </div>
      {intiface && (
        <div className="grid grid-cols-[minmax(8rem,1fr)_minmax(8rem,2fr)] items-center gap-x-4">
          <TooltipLabel
            text="Intiface address"
            tooltip={TOOLTIP.IntifaceAddress}
          />
          <input
            className="theme-input"
            aria-label="Intiface address"
            placeholder="e.g. 192.168.1.20:12345"
            spellCheck={false}
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            onBlur={saveAddress}
            onKeyDown={onKeyDown}
          />
        </div>
      )}
      {!choosingIntiface && (
        <div
          className={cn(
            "text-right text-xs",
            status.state === "Connected" && "text-theme-success",
            status.state === "Connecting" && "text-theme-muted",
            status.state === "Failed" && "text-theme-danger",
          )}
        >
          {status.state === "Connected"
            ? state.connection.mode === "Intiface"
              ? "Connected to Intiface"
              : "Bluetooth ready"
            : status.state === "Connecting"
              ? "Connecting…"
              : status.error}
        </div>
      )}
      {choosingIntiface && (
        <div className="text-right text-xs text-theme-muted">
          Enter the address Intiface shows, then press Enter
        </div>
      )}
    </div>
  );
}
