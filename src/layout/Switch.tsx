import ThemeIcon from "@/theme/ThemeIcon";
import * as SwitchPrimitive from "@radix-ui/react-switch";
import { SwitchProps } from "@radix-ui/react-switch";
import { cn } from "../lib/utils";

const themeColors = {
  // Default switch: sleepy lilac when off, pink when on
  "white-cyan": {
    inactive: "bg-theme-border",
    active: "bg-theme-primary",
    paw: "text-theme-primary",
  },
  // App on/off switch: rose when off, mint when on
  "red-green": {
    inactive: "bg-theme-danger",
    active: "bg-theme-success",
    paw: "text-theme-success",
  },
};

const sizes = {
  small: {
    container: "h-4 w-8 border-2",
    switch: "h-3 w-3",
    translateLeft: "translate-x-4",
    paw: null,
  },
  medium: {
    container: "h-6 w-12 border-2",
    switch: "h-5 w-5",
    translateLeft: "translate-x-6",
    paw: "h-3 w-3",
  },
  large: {
    container: "h-10 w-20 border-4",
    switch: "h-8 w-8",
    translateLeft: "translate-x-10",
    paw: "h-5 w-5",
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
  const selectedSize = sizes[size];
  return (
    <SwitchPrimitive.Root
      {...props}
      className={cn(
        props.checked ? selectedTheme.active : selectedTheme.inactive,
        selectedSize.container,
        "relative inline-flex cursor-pointer rounded-full border-transparent transition-all duration-200 ease-in-out disabled:cursor-not-allowed disabled:bg-theme-raised",
      )}
    >
      <SwitchPrimitive.Thumb
        className={cn(
          props.checked ? selectedSize.translateLeft : "translate-x-0",
          selectedSize.switch,
          props.disabled ? "bg-theme-border" : "bg-theme-text",
          "flex items-center justify-center rounded-full shadow transition duration-200 ease-in-out",
        )}
      >
        {selectedSize.paw && !props.disabled && (
          <ThemeIcon
            className={cn(
              selectedSize.paw,
              "transition-opacity duration-200",
              props.checked ? selectedTheme.paw : "text-theme-muted",
            )}
            strokeWidth={2.5}
          />
        )}
      </SwitchPrimitive.Thumb>
    </SwitchPrimitive.Root>
  );
}
