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
  const color = props?.accent && "from-theme-success to-theme-info";
  const thumbClass = cn(
    "block h-4 w-4 rounded-full border-2 bg-theme-text shadow transition-transform focus:outline-none focus-visible:ring-2 focus-visible:ring-theme-primary/50",
    props.disabled
      ? "border-theme-border"
      : "border-theme-primary hover:scale-110",
  );
  // This is really dumb right now
  // Hopefully RadixPopper "updatePositionStrategy" = "always" will be added to tooltip
  return (
    <SliderPrimitive.Root
      className={cn(
        "relative flex items-center",
        !props.disabled && "cursor-ew-resize",
      )}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      {...props}
      aria-label="Slider"
    >
      <SliderPrimitive.Track className="relative flex h-2 flex-grow rounded-full bg-theme-background outline outline-1 outline-theme-border">
        <SliderPrimitive.Range
          className={cn(
            "absolute h-full rounded-full bg-gradient-to-r from-theme-primary to-theme-secondary transition-all data-[disabled]:from-theme-border data-[disabled]:to-theme-border",
            color,
          )}
        />
      </SliderPrimitive.Track>
      {first != undefined && (
        <TooltipPrimitive.Provider>
          <TooltipPrimitive.Root open={hover}>
            <TooltipPrimitive.Trigger asChild>
              <SliderPrimitive.Thumb className={thumbClass} />
            </TooltipPrimitive.Trigger>
            <TooltipPrimitive.Portal>
              <TooltipPrimitive.Content
                className="max-w-md rounded-full bg-theme-secondary px-3 text-sm font-semibold text-theme-on-accent shadow-lg"
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
              <SliderPrimitive.Thumb className={thumbClass} />
            </TooltipPrimitive.Trigger>
            <TooltipPrimitive.Portal>
              <TooltipPrimitive.Content
                className="max-w-md rounded-full bg-theme-secondary px-3 text-sm font-semibold text-theme-on-accent shadow-lg"
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
