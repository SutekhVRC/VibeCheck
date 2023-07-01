import * as RadixSlider from "@radix-ui/react-slider";
import * as RadixTooltip from "@radix-ui/react-tooltip";
import { cn, round0 } from "../utils";
import { useState } from "react";

export default function Slider(
  props: RadixSlider.SliderProps & { multiply?: number }
) {
  const [hover, setHover] = useState(false);
  const first = props.value?.at(0);
  const second = props.value?.at(1);
  // This is really dumb right now
  // Hopefully RadixPopper "updatePositionStrategy" = "always" will be added to tooltip
  return (
    <RadixSlider.Root
      className={cn(
        "relative flex items-center",
        !props.disabled && "cursor-ew-resize"
      )}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      {...props}
      aria-label="Slider"
    >
      <RadixSlider.Track className="relative bg-gray-700 flex-grow rounded-full h-2">
        <RadixSlider.Range className="absolute bg-gray-100 rounded-full h-full data-[disabled]:bg-gray-600 transition-all" />
      </RadixSlider.Track>
      {first != undefined && (
        <RadixTooltip.Provider>
          <RadixTooltip.Root open={hover}>
            <RadixTooltip.Trigger asChild>
              <RadixSlider.Thumb />
            </RadixTooltip.Trigger>
            <RadixTooltip.Portal>
              <RadixTooltip.Content
                className="rounded-md text-gray-50 bg-gray-600 pl-3 pr-3 max-w-md"
                sideOffset={10}
                key={first} // force update with key
              >
                {props.multiply ? round0.format(first * props.multiply) : first}
              </RadixTooltip.Content>
            </RadixTooltip.Portal>
          </RadixTooltip.Root>
        </RadixTooltip.Provider>
      )}
      {second != undefined && (
        <RadixTooltip.Provider>
          <RadixTooltip.Root open={hover}>
            <RadixTooltip.Trigger asChild>
              <RadixSlider.Thumb />
            </RadixTooltip.Trigger>
            <RadixTooltip.Portal>
              <RadixTooltip.Content
                className="rounded-md text-gray-50 bg-gray-600 pl-3 pr-3 max-w-md"
                sideOffset={10}
                key={second} // force update with key
              >
                {props.multiply
                  ? round0.format(second * props.multiply)
                  : second}
              </RadixTooltip.Content>
            </RadixTooltip.Portal>
          </RadixTooltip.Root>
        </RadixTooltip.Provider>
      )}
    </RadixSlider.Root>
  );
}
