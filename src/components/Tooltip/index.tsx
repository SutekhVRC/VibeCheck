import type { ReactNode } from "react";
import * as Tooltip from "@radix-ui/react-tooltip";

export default function ({
  children,
  text,
}: {
  children: ReactNode;
  text: string;
}) {
  return (
    <Tooltip.Provider delayDuration={100}>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>{children}</Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            className="rounded-md text-gray-50 bg-gray-600 pl-3 pr-3"
            sideOffset={5}
          >
            {text}
            <Tooltip.Arrow className="fill-gray-600" />
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
}
