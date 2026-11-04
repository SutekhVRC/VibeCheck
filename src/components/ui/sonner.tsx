import { mix, withAlpha } from "@/theme/color";
import ThemeIcon from "@/theme/ThemeIcon";
import { useTheme } from "@/theme/ThemeProvider";
import type { ThemeColors } from "@/theme/types";
import { Bell, HeartCrack, Sparkles } from "lucide-react";
import type { CSSProperties } from "react";
import { Toaster as Sonner } from "sonner";

type ToasterProps = React.ComponentProps<typeof Sonner>;

/*
 * Sonner styles toasts through CSS variables on the toaster element.
 * Setting them inline overrides sonner's built-in palettes, so every toast
 * type follows the active theme. Each status gets its colour for text and
 * border, over a background tinted slightly towards that colour.
 */
function toastVars(c: Required<ThemeColors>, sharp: boolean): CSSProperties {
  const status = (prefix: string, color: string) => ({
    [`--${prefix}-bg`]: mix(c.surface, color, 0.08),
    [`--${prefix}-border`]: withAlpha(color, 0.55),
    [`--${prefix}-text`]: color,
  });
  return {
    "--normal-bg": c.surface,
    "--normal-border": withAlpha(c.secondary, 0.45),
    "--normal-text": c.text,
    ...status("success", c.success),
    ...status("info", c.info),
    ...status("warning", c.warning),
    ...status("error", c.danger),
    "--border-radius": sharp ? "0" : "1.25rem",
    // Sonner sets its own font stack; use the theme's font instead
    fontFamily: "inherit",
  } as CSSProperties;
}

const Toaster = ({ ...props }: ToasterProps) => {
  const { theme } = useTheme();
  return (
    <Sonner
      theme={theme.scheme}
      richColors
      style={toastVars(theme.colors, theme.decorations.corners === "sharp")}
      icons={{
        success: <ThemeIcon className="h-4 w-4" />,
        info: <Sparkles className="h-4 w-4" />,
        warning: <Bell className="h-4 w-4" />,
        error: <HeartCrack className="h-4 w-4" />,
      }}
      toastOptions={{
        style: {
          // Error messages put backend details on a second line with "\n"
          whiteSpace: "pre-line",
          // Same look as other popups for the theme's shadow style
          boxShadow: "var(--theme-shadow-pop)",
        },
      }}
      {...props}
    />
  );
};

export { Toaster };
