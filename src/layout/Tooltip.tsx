import { INVOKE, TOOLTIP } from "@/data/constants";
import { ObjectValues } from "@/utils";
import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import { invoke } from "@tauri-apps/api";
import type { ReactNode } from "react";

export default function Tooltip({
  children,
  text,
  delay = 150,
  asChild = true,
}: {
  children: ReactNode;
  text: string;
  delay?: number;
  asChild?: boolean;
}) {
  return (
    <TooltipPrimitive.Provider delayDuration={delay}>
      <TooltipPrimitive.Root>
        <TooltipPrimitive.Trigger asChild={asChild}>
          {children}
        </TooltipPrimitive.Trigger>
        <TooltipPrimitive.Portal>
          <TooltipPrimitive.Content
            className="pointer-events-none z-50 max-w-md cursor-pointer rounded-md bg-zinc-600 pl-3 pr-3 text-zinc-50"
            sideOffset={5}
          >
            {text}
            <TooltipPrimitive.Arrow className="fill-zinc-600" />
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

export function TooltipLabel({ text, tooltip }: TooltipProps) {
  const tooltipText = typeof tooltip == "string" ? tooltip : tooltip.text;
  const link =
    typeof tooltip != "string" && tooltip.link ? tooltip.link : undefined;
  if (link) {
    return (
      <Tooltip text={tooltipText}>
        <label
          onClick={async () => {
            await invoke(INVOKE.OPEN_BROWSER, { link });
          }}
          className="cursor-pointer select-none justify-self-start underline"
        >
          {text}
        </label>
      </Tooltip>
    );
  } else {
    return (
      <Tooltip text={tooltipText}>
        <label className="cursor-help justify-self-start">{text}</label>
      </Tooltip>
    );
  }
}
