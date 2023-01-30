import type { ReactNode } from "react";
import * as RadixTooltip from "@radix-ui/react-tooltip";

export default function Tooltip({
  children,
  text,
  delay = 150,
}: {
  children: ReactNode;
  text: string;
  delay?: number;
}) {
  return (
    <RadixTooltip.Provider delayDuration={delay}>
      <RadixTooltip.Root>
        <RadixTooltip.Trigger asChild>{children}</RadixTooltip.Trigger>
        <RadixTooltip.Portal>
          <RadixTooltip.Content
            className="rounded-md text-gray-50 bg-gray-600 pl-3 pr-3 max-w-md z-50"
            sideOffset={5}
          >
            {text}
            <RadixTooltip.Arrow className="fill-gray-600" />
          </RadixTooltip.Content>
        </RadixTooltip.Portal>
      </RadixTooltip.Root>
    </RadixTooltip.Provider>
  );
}
