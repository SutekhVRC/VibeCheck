import { INVOKE, TOOLTIP } from "@/data/constants";
import { ObjectValues } from "@/utils";
import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import { invoke } from "@tauri-apps/api/core";
import { BookOpen } from "lucide-react";
import type { ReactNode } from "react";
import { toast } from "sonner";

export default function Tooltip({
  children,
  text,
  delay = 150,
  asChild = true,
  side = "top",
}: {
  children: ReactNode;
  text: string;
  delay?: number;
  asChild?: boolean;
  /** Which side of the element the tooltip opens on */
  side?: "top" | "right" | "bottom" | "left";
}) {
  return (
    <TooltipPrimitive.Provider delayDuration={delay}>
      <TooltipPrimitive.Root>
        <TooltipPrimitive.Trigger asChild={asChild}>
          {children}
        </TooltipPrimitive.Trigger>
        <TooltipPrimitive.Portal>
          <TooltipPrimitive.Content
            side={side}
            className="pointer-events-none z-50 max-w-md cursor-pointer rounded-2xl border border-theme-border bg-theme-raised px-3 py-1 text-sm text-theme-text shadow-theme-tooltip"
            sideOffset={5}
          >
            {text}
            <TooltipPrimitive.Arrow className="fill-theme-raised" />
          </TooltipPrimitive.Content>
        </TooltipPrimitive.Portal>
      </TooltipPrimitive.Root>
    </TooltipPrimitive.Provider>
  );
}

type TooltipProps = {
  text: string;
  tooltip: string | ObjectValues<typeof TOOLTIP>;
};

/*
 * A setting's label with its explanation in a hover tooltip. Labels that
 * have a docs page get a small book icon that opens it, so every label looks
 * the same and only the icon is clickable (no accidental browser opens).
 */
export function TooltipLabel({ text, tooltip }: TooltipProps) {
  const tooltipText = typeof tooltip == "string" ? tooltip : tooltip.text;
  const link =
    typeof tooltip != "string" && tooltip.link ? tooltip.link : undefined;

  async function openDocs() {
    try {
      await invoke(INVOKE.OPEN_BROWSER, { link });
    } catch (e) {
      toast.error(`Could not open browser\n${JSON.stringify(e)}`);
    }
  }

  return (
    <span className="inline-flex items-center gap-1.5 justify-self-start">
      <Tooltip text={tooltipText}>
        <label className="cursor-help select-none text-theme-text/90">
          {text}
        </label>
      </Tooltip>
      {link && (
        <Tooltip text="Open the guide for this setting">
          <button
            type="button"
            onClick={openDocs}
            aria-label={`Open the guide for ${text}`}
            className="rounded-full p-0.5 text-theme-muted transition-colors hover:bg-theme-primary/15 hover:text-theme-primary"
          >
            <BookOpen className="h-3.5 w-3.5" />
          </button>
        </Tooltip>
      )}
    </span>
  );
}
