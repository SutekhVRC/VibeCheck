import * as SliderPrimitive from "@radix-ui/react-slider";
import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import { round0 } from "../utils";
import { cn } from "../lib/utils";
import { useState } from "react";

export default function Slider(
  props: SliderPrimitive.SliderProps & { multiply?: number }
) {
  const [hover, setHover] = useState(false);
  const first = props.value?.at(0);
  const second = props.value?.at(1);
  // This is really dumb right now
  // Hopefully RadixPopper "updatePositionStrategy" = "always" will be added to tooltip
  return (
    <SliderPrimitive.Root
      className={cn(
        "relative flex items-center",
        !props.disabled && "cursor-ew-resize"
      )}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      {...props}
      aria-label="Slider"
    >
      <SliderPrimitive.Track className="relative bg-gray-700 flex-grow rounded-full h-2">
        <SliderPrimitive.Range className="absolute bg-gray-100 rounded-full h-full data-[disabled]:bg-gray-600 transition-all" />
      </SliderPrimitive.Track>
      {first != undefined && (
        <TooltipPrimitive.Provider>
          <TooltipPrimitive.Root open={hover}>
            <TooltipPrimitive.Trigger asChild>
              <SliderPrimitive.Thumb />
            </TooltipPrimitive.Trigger>
            <TooltipPrimitive.Portal>
              <TooltipPrimitive.Content
                className="rounded-md text-gray-50 bg-gray-600 pl-3 pr-3 max-w-md"
                sideOffset={10}
                key={first} // force update with key
              >
                {props.multiply ? round0.format(first * props.multiply) : first}
              </TooltipPrimitive.Content>
            </TooltipPrimitive.Portal>
          </TooltipPrimitive.Root>
        </TooltipPrimitive.Provider>
      )}
      {second != undefined && (
        <TooltipPrimitive.Provider>
          <TooltipPrimitive.Root open={hover}>
            <TooltipPrimitive.Trigger asChild>
              <SliderPrimitive.Thumb />
            </TooltipPrimitive.Trigger>
            <TooltipPrimitive.Portal>
              <TooltipPrimitive.Content
                className="rounded-md text-gray-50 bg-gray-600 pl-3 pr-3 max-w-md"
                sideOffset={10}
                key={second} // force update with key
              >
                {props.multiply
                  ? round0.format(second * props.multiply)
                  : second}
              </TooltipPrimitive.Content>
            </TooltipPrimitive.Portal>
          </TooltipPrimitive.Root>
        </TooltipPrimitive.Provider>
      )}
    </SliderPrimitive.Root>
  );
}
