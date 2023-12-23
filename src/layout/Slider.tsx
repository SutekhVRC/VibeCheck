import * as SliderPrimitive from "@radix-ui/react-slider";
import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import { useState } from "react";
import { cn } from "../lib/utils";
import { round0 } from "../utils";

type SliderProps = {
  multiply?: number;
  accent?: boolean;
};

export default function Slider(
  props: SliderPrimitive.SliderProps & SliderProps,
) {
  const [hover, setHover] = useState(false);
  const first = props.value?.at(0);
  const second = props.value?.at(1);
  const color = props?.accent && "bg-cyan-500";
  // This is really dumb right now
  // Hopefully RadixPopper "updatePositionStrategy" = "always" will be added to tooltip
  return (
    <SliderPrimitive.Root
      className={cn(
        "relative items-center flex",
        !props.disabled && "cursor-ew-resize",
      )}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      {...props}
      aria-label="Slider"
    >
      <SliderPrimitive.Track className="relative flex bg-zinc-700 flex-grow rounded-full h-2">
        <SliderPrimitive.Range
          className={cn(
            "absolute bg-zinc-100 rounded-full h-full data-[disabled]:bg-zinc-600 transition-all",
            color,
          )}
        />
      </SliderPrimitive.Track>
      {first != undefined && (
        <TooltipPrimitive.Provider>
          <TooltipPrimitive.Root open={hover}>
            <TooltipPrimitive.Trigger asChild>
              <SliderPrimitive.Thumb />
            </TooltipPrimitive.Trigger>
            <TooltipPrimitive.Portal>
              <TooltipPrimitive.Content
                className="rounded-md text-zinc-50 bg-zinc-600 pl-3 pr-3 max-w-md"
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
                className="rounded-md text-zinc-50 bg-zinc-600 pl-3 pr-3 max-w-md"
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
