import { Sparkles } from "lucide-react";
import { hexToRgb } from "./color";
import { logoGradient } from "./logo";
import type { EarStyle } from "./styles";
import type { Theme, ThemeDefinition } from "./types";

/** `true` is shorthand for cat ears, `false`/missing for none */
function earStyle(ears: boolean | EarStyle | undefined): EarStyle {
  if (ears === true) return "cat";
  if (ears === false || ears === undefined) return "none";
  return ears;
}

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
      // Outlines -> text, "VIBE" -> secondary, lips -> primary, with the
      // logo's shading kept (see logo.ts)
      stops:
        def.logo?.stops ??
        logoGradient({
          outline: colors.text,
          vibe: colors.secondary,
          lips: colors.primary,
          background: colors.background,
        }),
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
      ears: earStyle(def.decorations?.ears),
      pattern: def.decorations?.pattern ?? "none",
      icon: def.decorations?.icon ?? Sparkles,
      corners: def.decorations?.corners ?? "round",
      shadows: def.decorations?.shadows ?? "glow",
      font: def.decorations?.font ?? "rounded",
    },
  };
}
