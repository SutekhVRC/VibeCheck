import { Sparkles } from "lucide-react";
import { hexToRgb } from "./color";
import type { Theme, ThemeDefinition } from "./types";

/**
 * Turns a theme file's definition into a complete Theme by filling in
 * defaults. Every theme file should `export default defineTheme({...})`.
 */
export function defineTheme(def: ThemeDefinition): Theme {
  const colors = {
    ...def.colors,
    onAccent: def.colors.onAccent ?? def.colors.background,
  };

  // Catch typos early: throws with the bad value if a colour isn't #rrggbb
  for (const value of Object.values(colors)) hexToRgb(value);

  const scanButton = def.text?.scanButton ?? "Look for toys";

  return {
    id: def.id,
    name: def.name,
    scheme: def.scheme ?? "dark",
    colors,
    logo: {
      stops: def.logo?.stops ?? [
        // Black outlines -> text colour so they stand out on the background
        { brightness: 0, color: colors.text },
        { brightness: 0.12, color: colors.text },
        // Purple "VIBE" -> secondary
        { brightness: 0.26, color: colors.secondary },
        { brightness: 0.4, color: colors.secondary },
        // Coral lips and "check" -> primary
        { brightness: 0.55, color: colors.primary },
        { brightness: 0.75, color: colors.primary },
        // Highlights -> white
        { brightness: 1, color: "#ffffff" },
      ],
      glow: def.logo?.glow === undefined ? colors.primary : def.logo.glow,
    },
    text: {
      scanButton,
      scanning: def.text?.scanning ?? "Scanning",
      noToysTitle: def.text?.noToysTitle ?? "No toys online yet",
      noToysHint:
        def.text?.noToysHint ?? `Press \u201c${scanButton}\u201d to find some`,
    },
    decorations: {
      ears: def.decorations?.ears ?? false,
      pattern: def.decorations?.pattern ?? "none",
      icon: def.decorations?.icon ?? Sparkles,
      corners: def.decorations?.corners ?? "round",
      shadows: def.decorations?.shadows ?? "glow",
      font: def.decorations?.font ?? "rounded",
    },
  };
}
