import {
  Popover,
  PopoverAnchor,
  PopoverContent,
} from "@/components/ui/popover";
import { INVOKE, TOOLTIP } from "@/data/constants";
import { Select } from "@/layout/Select";
import { TooltipLabel } from "@/layout/Tooltip";
import { errorText } from "@/lib/errors";
import { cn } from "@/lib/utils";
import type { FeToyConnectionState } from "@bindings/FeToyConnectionState";
import type { SavedAddress } from "@bindings/SavedAddress";
import type { ToyConnection as Connection } from "@bindings/ToyConnection";
import { invoke } from "@tauri-apps/api/core";
import { Check, ChevronDown, Pencil, X } from "lucide-react";
import { KeyboardEvent, useEffect, useState } from "react";
import { toast } from "sonner";

/*
 * Settings > Toys > Toy connection: this computer's Bluetooth (VibeCheck's
 * own toy service), or Intiface Central on this computer or a phone, for
 * computers without Bluetooth (src-tauri util/toy_service.rs). Switching
 * applies at once: toys on the old connection disconnect.
 *
 * Intiface addresses that connected are remembered (newest first) and
 * offered in a list under the address field, where they can be named
 * ("Phone") or removed, so nobody has to type them again.
 */

/** An address as people write it: without ws:// */
const shortAddress = (address: string) => address.replace(/^ws:\/\//, "");

/** What a saved address is called: its name, else the address */
const savedLabel = (s: SavedAddress) => s.name || shortAddress(s.address);

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
  }, []);

  // The saved address, when there is one. Only a change to it updates the
  // field: the status refresh every 2 s mustn't overwrite what's typed
  const saved = state?.connection;
  const savedAddress = saved?.mode === "Intiface" ? saved.address : null;
  useEffect(() => {
    if (savedAddress != null) setAddress(savedAddress);
  }, [savedAddress]);

  const savedList = state?.saved ?? [];

  /** Saves edits to the remembered addresses (rename, remove) */
  async function saveList(addresses: SavedAddress[]) {
    try {
      const done = await invoke<SavedAddress[]>(
        INVOKE.SET_SAVED_INTIFACE_ADDRESSES,
        { addresses },
      );
      setState((s) => (s ? { ...s, saved: done } : s));
    } catch (e) {
      toast.error(errorText(e));
    }
  }

  async function save(connection: Connection) {
    try {
      const done = await invoke<Connection>(INVOKE.SET_TOY_CONNECTION, {
        connection,
      });
      setChoosingIntiface(false);
      setState((s) =>
        s ? { ...s, connection: done, status: { state: "Connecting" } } : s,
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
    <div className="flex flex-col gap-2">
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
              // Straight back to the last address used, if there is one;
              // otherwise saved once one is entered
              const last = state.saved[0];
              if (last) save({ mode: "Intiface", address: last.address });
              else setChoosingIntiface(true);
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
          <AddressField
            value={address}
            onChange={setAddress}
            onCommit={saveAddress}
            onKeyDown={onKeyDown}
            saved={savedList}
            current={savedAddress}
            onPick={(picked) => {
              setAddress(picked);
              if (picked !== savedAddress)
                save({ mode: "Intiface", address: picked });
            }}
            onSaveList={saveList}
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
              ? `Connected to Intiface${connectedName(savedList, savedAddress)}`
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

/** " (Phone)" when the connected address has a name */
function connectedName(saved: SavedAddress[], address: string | null) {
  const name = saved.find((s) => s.address === address)?.name;
  return name ? ` (${name})` : "";
}

/**
 * The Intiface address: typed, or picked from the addresses that connected
 * before (the arrow), which can be named or removed there.
 */
function AddressField({
  value,
  onChange,
  onCommit,
  onKeyDown,
  saved,
  current,
  onPick,
  onSaveList,
}: {
  value: string;
  onChange: (value: string) => void;
  onCommit: () => void;
  onKeyDown: (e: KeyboardEvent<HTMLInputElement>) => void;
  saved: SavedAddress[];
  /** The address in use, as saved (with ws://) */
  current: string | null;
  onPick: (address: string) => void;
  onSaveList: (addresses: SavedAddress[]) => void;
}) {
  const [open, setOpen] = useState(false);
  // The address being renamed, and the name typed so far
  const [renaming, setRenaming] = useState<string | null>(null);
  const [name, setName] = useState("");

  function rename(address: string) {
    onSaveList(
      saved.map((s) =>
        s.address === address ? { ...s, name: name.trim() } : s,
      ),
    );
    setRenaming(null);
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverAnchor asChild>
        <div className="theme-input flex items-center gap-1 py-0 pr-1">
          <input
            className="min-w-0 flex-1 bg-transparent py-1 outline-none"
            aria-label="Intiface address"
            placeholder="e.g. 192.168.1.20:12345"
            spellCheck={false}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            onBlur={onCommit}
            onKeyDown={onKeyDown}
          />
          {saved.length > 0 && (
            <button
              type="button"
              onClick={() => setOpen((o) => !o)}
              aria-label="Addresses used before"
              title="Addresses used before"
              aria-expanded={open}
              className="rounded p-0.5 text-theme-secondary hover:bg-theme-raised"
            >
              <ChevronDown
                className={cn(
                  "h-4 w-4 transition-transform",
                  open && "rotate-180",
                )}
              />
            </button>
          )}
        </div>
      </PopoverAnchor>
      <PopoverContent
        align="end"
        className="w-[var(--radix-popover-trigger-width)] min-w-[18rem] rounded-xl border-theme-border bg-theme-raised p-1 text-sm text-theme-text"
        onOpenAutoFocus={(e) => e.preventDefault()}
      >
        <div className="px-2 pb-1 pt-0.5 text-xs text-theme-muted">
          Used before
        </div>
        {saved.map((s) =>
          renaming === s.address ? (
            <div
              key={s.address}
              className="flex items-center gap-1 px-1 py-0.5"
            >
              <input
                autoFocus
                className="theme-input min-w-0 flex-1 py-0.5"
                aria-label="Name"
                placeholder="e.g. Phone"
                value={name}
                onChange={(e) => setName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") rename(s.address);
                  if (e.key === "Escape") setRenaming(null);
                }}
              />
              <button
                type="button"
                onClick={() => rename(s.address)}
                aria-label="Save name"
                className="rounded p-1 text-theme-success hover:bg-theme-surface"
              >
                <Check className="h-3.5 w-3.5" />
              </button>
            </div>
          ) : (
            <div
              key={s.address}
              className={cn(
                "group flex items-center gap-2 rounded-lg px-2 py-1 hover:bg-theme-surface",
                s.address === current && "text-theme-primary",
              )}
            >
              <button
                type="button"
                onClick={() => {
                  onPick(s.address);
                  setOpen(false);
                }}
                className="flex min-w-0 flex-1 items-baseline gap-2 text-left"
              >
                <span className="truncate">{savedLabel(s)}</span>
                {s.name && (
                  <span className="truncate text-xs text-theme-muted">
                    {shortAddress(s.address)}
                  </span>
                )}
              </button>
              <button
                type="button"
                onClick={() => {
                  setRenaming(s.address);
                  setName(s.name);
                }}
                aria-label={`Name ${savedLabel(s)}`}
                title="Name it"
                className="rounded p-1 text-theme-muted hover:text-theme-text"
              >
                <Pencil className="h-3.5 w-3.5" />
              </button>
              <button
                type="button"
                onClick={() =>
                  onSaveList(saved.filter((x) => x.address !== s.address))
                }
                aria-label={`Forget ${savedLabel(s)}`}
                title="Forget it"
                className="rounded p-1 text-theme-muted hover:text-theme-danger"
              >
                <X className="h-3.5 w-3.5" />
              </button>
            </div>
          ),
        )}
        <div className="px-2 pb-0.5 pt-1 text-xs text-theme-muted">
          A new address: type it in the field and press Enter. It&apos;s added
          here once it connects.
        </div>
      </PopoverContent>
    </Popover>
  );
}
