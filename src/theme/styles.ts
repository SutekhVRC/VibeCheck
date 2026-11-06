/*
 * Shape, shadow and font options a theme can pick in `decorations`.
 * ThemeProvider writes the chosen values into CSS variables / attributes.
 */

/** Ears on top of the main panels */
export type EarStyle = "none" | "cat" | "kitten" | "fox";

export type CornerStyle = "round" | "sharp";

export type ShadowStyle = "glow" | "neon" | "hard" | "none";

export type FontStyle = "rounded" | "mono" | "sans";

/** CSS box-shadow for each shadowed element (and the drop-shadow for images) */
type ShadowSet = {
  panel: string;
  button: string;
  pop: string;
  tooltip: string;
  /** Used inside CSS drop-shadow(), so it can't be "none" */
  drop: string;
};

// box-shadow values must stay valid inside a list (Tailwind adds ring
// shadows in front), so "no shadow" is written as a transparent shadow.
const NO_SHADOW = "0 0 #0000";

export const shadowStyles: Record<ShadowStyle, ShadowSet> = {
  glow: {
    panel: "0 8px 30px -12px rgb(var(--theme-secondary) / 0.35)",
    button: "0 4px 14px -4px rgb(var(--theme-primary) / 0.6)",
    pop: "0 10px 30px -8px rgb(var(--theme-secondary) / 0.45)",
    tooltip: "0 6px 20px -6px rgb(var(--theme-secondary) / 0.45)",
    drop: "0 0 25px rgb(var(--theme-primary) / 0.35)",
  },
  neon: {
    panel:
      "0 0 0 1px rgb(var(--theme-primary) / 0.35), 0 0 28px -6px rgb(var(--theme-primary) / 0.55)",
    button: "0 0 18px -2px rgb(var(--theme-primary) / 0.85)",
    pop: "0 0 0 1px rgb(var(--theme-secondary) / 0.5), 0 0 26px -4px rgb(var(--theme-secondary) / 0.6)",
    tooltip: "0 0 16px -2px rgb(var(--theme-secondary) / 0.7)",
    drop: "0 0 18px rgb(var(--theme-primary) / 0.6)",
  },
  hard: {
    panel: "6px 6px 0 0 rgb(var(--theme-primary))",
    button: "3px 3px 0 0 rgb(var(--theme-text))",
    pop: "4px 4px 0 0 rgb(var(--theme-primary))",
    tooltip: "3px 3px 0 0 rgb(var(--theme-primary))",
    drop: "4px 4px 0 rgb(var(--theme-primary))",
  },
  none: {
    panel: NO_SHADOW,
    button: NO_SHADOW,
    pop: NO_SHADOW,
    tooltip: NO_SHADOW,
    drop: "0 0 0 transparent",
  },
};

export const fontStacks: Record<FontStyle, string> = {
  rounded: `"Nunito", "Quicksand", "Varela Round", ui-rounded, "SF Pro Rounded", "Segoe UI", system-ui, sans-serif`,
  mono: `"JetBrains Mono", "Fira Code", "Cascadia Mono", ui-monospace, "DejaVu Sans Mono", monospace`,
  sans: `"Inter", "Segoe UI", system-ui, "Noto Sans", sans-serif`,
};
