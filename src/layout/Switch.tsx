import type { SwitchProps } from "@radix-ui/react-switch";
import * as SwitchPrimitive from "@radix-ui/react-switch";
import { cn } from "../lib/utils";

const themeColors = {
  "white-cyan": {
    inactive: "bg-zinc-300",
    active: "bg-cyan-600",
  },
  "red-green": {
    inactive: "bg-red-600",
    active: "bg-green-600",
  },
};

const sizes = {
  small: {
    container: "h-4 w-8 border-2",
    switch: "h-3 w-3",
    translateLeft: "translate-x-4",
  },
  medium: {
    container: "h-6 w-12 border-2",
    switch: "h-5 w-5",
    translateLeft: "translate-x-6",
  },
  large: {
    container: "h-10 w-20 border-4",
    switch: "h-8 w-8",
    translateLeft: "translate-x-10",
  },
};

export default function Switch(
  props: SwitchProps & {
    theme?: keyof typeof themeColors;
    size?: keyof typeof sizes;
  },
) {
  const theme = props.theme ?? "white-cyan";
  const size = props.size ?? "medium";
  const selectedTheme = themeColors[theme];
  return (
    <SwitchPrimitive.Root
      {...props}
      className={cn(
        props.checked ? selectedTheme.active : selectedTheme.inactive,
        sizes[size].container,
        "relative inline-flex cursor-pointer rounded-full border-transparent transition-all duration-200 ease-in-out disabled:cursor-not-allowed disabled:bg-zinc-700",
      )}
    >
      <SwitchPrimitive.Thumb
        className={cn(
          props.checked ? sizes[size].translateLeft : "translate-x-0",
          sizes[size].switch,
          props.disabled ? "bg-zinc-600" : "bg-white",
          "inline-block rounded-full transition duration-200 ease-in-out",
        )}
      />
    </SwitchPrimitive.Root>
  );
}
