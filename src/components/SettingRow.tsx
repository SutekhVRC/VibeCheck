import { TOOLTIP } from "@/data/constants";
import Tooltip, { TooltipLabel } from "@/layout/Tooltip";
import { ObjectValues } from "@/utils";
import { ArrowRightLeft } from "lucide-react";
import { ReactNode } from "react";

/*
 * One setting: label on the left, a small control (switch, number) on the
 * right, and optionally a full-width control (slider, dropdowns) underneath.
 * Replaces the old four-column grid so rows line up and nothing sits in an
 * empty cell.
 */
export function SettingRow({
  label,
  tooltip,
  flipped,
  extra,
  control,
  children,
}: {
  label: string;
  tooltip?: string | ObjectValues<typeof TOOLTIP>;
  /** Shows the "input is flipped" marker next to the label */
  flipped?: boolean;
  /** Small extra next to the label, e.g. a copy button */
  extra?: ReactNode;
  /** Right-aligned control on the label line */
  control?: ReactNode;
  /** Full-width control under the label line */
  children?: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-2 py-2.5">
      <div className="flex min-h-[1.75rem] items-center justify-between gap-3">
        <div className="flex min-w-0 items-center gap-2">
          {tooltip ? (
            <TooltipLabel text={label} tooltip={tooltip} />
          ) : (
            <span>{label}</span>
          )}
          {flipped && (
            <Tooltip text="Input is flipped">
              <ArrowRightLeft className="h-4 w-4 shrink-0 text-theme-muted" />
            </Tooltip>
          )}
          {extra}
        </div>
        {control && (
          <div className="flex shrink-0 items-center gap-1">{control}</div>
        )}
      </div>
      {children && <div className="w-full">{children}</div>}
    </div>
  );
}

/** A titled group of SettingRows with thin dividers between them */
export function SettingsGroup({
  title,
  children,
}: {
  title?: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className="text-left text-sm">
      {title && (
        <div className="mb-0.5 mt-1 text-xs font-semibold uppercase tracking-wider text-theme-muted">
          {title}
        </div>
      )}
      <div className="divide-y divide-theme-border/40">{children}</div>
    </div>
  );
}
