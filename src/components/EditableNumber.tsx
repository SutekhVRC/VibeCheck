import { KeyboardEvent, useEffect, useRef, useState } from "react";
import { cn } from "../lib/utils";

type EditableNumberProps = {
  /** Value in the slider's own units (e.g. 0-1 for levels) */
  value: number;
  min: number;
  max: number;
  step: number;
  /**
   * Multiplier between the stored value and what the user sees and types.
   * Matches the Slider `multiply` prop, e.g. 100 shows 0-1 levels as 0-100.
   */
  scale?: number;
  /** Text shown after the number, e.g. "hz" */
  suffix?: string;
  disabled?: boolean;
  /** Called with the clamped, step-snapped value in the slider's own units */
  onCommit: (value: number) => void;
  label?: string;
};

/** Number of decimals needed to show a step, e.g. 0.05 -> 2, 1 -> 0 */
function decimalsFor(step: number) {
  const text = step.toString();
  const dot = text.indexOf(".");
  return dot === -1 ? 0 : text.length - dot - 1;
}

/*
 * A slider value that can be clicked and typed into.
 * Precise number entry is much easier than dragging a slider with a VR
 * laser pointer. Enter or clicking away applies, Escape cancels.
 */
export default function EditableNumber({
  value,
  min,
  max,
  step,
  scale = 1,
  suffix = "",
  disabled = false,
  onCommit,
  label,
}: EditableNumberProps) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  // Set when Escape is pressed so the following blur doesn't apply the draft
  const cancelledRef = useRef(false);

  const displayDecimals = decimalsFor(step * scale);
  const format = (v: number) =>
    (v * scale).toLocaleString("en-US", {
      minimumFractionDigits: 0,
      maximumFractionDigits: displayDecimals,
      useGrouping: false,
    });

  useEffect(() => {
    if (editing) inputRef.current?.select();
  }, [editing]);

  function startEditing() {
    if (disabled) return;
    cancelledRef.current = false;
    setDraft(format(value));
    setEditing(true);
  }

  function apply() {
    setEditing(false);
    if (cancelledRef.current) return;
    const typed = parseFloat(draft.replace(",", "."));
    if (!Number.isFinite(typed)) return; // invalid input: keep the old value

    const clamped = Math.min(max, Math.max(min, typed / scale));
    const snapped = Math.round((clamped - min) / step) * step + min;
    // Trim floating point noise like 0.30000000000000004
    const result = Math.min(
      max,
      Number(snapped.toFixed(decimalsFor(step) + 2)),
    );
    if (result !== value) onCommit(result);
  }

  function onKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") {
      inputRef.current?.blur();
    } else if (e.key === "Escape") {
      cancelledRef.current = true;
      inputRef.current?.blur();
    }
  }

  if (editing) {
    return (
      <span className="inline-flex shrink-0 items-center justify-end gap-0.5 whitespace-nowrap">
        <input
          ref={inputRef}
          className="theme-input w-16 px-1.5 text-right"
          inputMode="decimal"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={apply}
          onKeyDown={onKeyDown}
          aria-label={label ?? "Value"}
        />
        {suffix && <span className="text-theme-muted">{suffix}</span>}
      </span>
    );
  }

  return (
    <button
      type="button"
      onClick={startEditing}
      disabled={disabled}
      title={disabled ? undefined : "Click to type a value"}
      aria-label={label ? `${label}: ${format(value)}` : undefined}
      className={cn(
        "shrink-0 whitespace-nowrap rounded-lg border border-dashed px-1.5 tabular-nums transition-colors",
        disabled
          ? "cursor-default border-transparent text-theme-muted"
          : "cursor-text border-theme-border hover:border-theme-primary hover:bg-theme-primary/10 hover:text-theme-primary",
      )}
    >
      {format(value)}
      {suffix}
    </button>
  );
}
