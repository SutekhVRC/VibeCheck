import type { LucideIcon } from "lucide-react";
import type { PatternName } from "./patterns";
import type { CornerStyle, FontStyle, ShadowStyle } from "./styles";

/**
 * The colour roles every theme fills in. Components only ever use these
 * roles (e.g. `bg-theme-primary`), never raw colours, so a theme is just a
 * choice of colour for each role.
 *
 * All colours are 6-digit hex strings like "#f7b2d9".
 */
export type ThemeColors = {
  /** Page background, and the darkest layer of the UI */
  background: string;
  /** Main panels (sidebar, settings pane) */
  surface: string;
  /** Boxes inside panels (feature options, tooltips, toy buttons) */
  raised: string;
  /** Borders and dividers */
  border: string;
  /** Main text */
  text: string;
  /** Secondary text, placeholders, offline toys */
  muted: string;
  /** Main accent: buttons, selected items, switches, slider fill */
  primary: string;
  /** Second accent: gradients with primary, glows, icons */
  secondary: string;
  /** "On" / good things: app enabled switch, update dot, success toasts */
  success: string;
  /** "Off" / bad things: app disabled switch, remove buttons, error toasts */
  danger: string;
  /** Links and info toasts */
  info: string;
  /** Warning toasts */
  warning: string;
  /**
   * Text drawn on top of primary/secondary-coloured fills (button labels,
   * slider value bubbles). Optional: defaults to `background`, which suits
   * dark themes with light accents. Light themes usually want a dark colour.
   */
  onAccent?: string;
};

/**
 * One stop of the logo gradient map.
 * Pixels of the original logo with this brightness (0 = black, 1 = white)
 * are recoloured to `color`. Colours in between are blended.
 */
export type LogoStop = {
  brightness: number;
  color: string;
};

/** Wording that changes with the theme's personality */
export type ThemeText = {
  /** Sidebar button that starts a Bluetooth scan. Default: "Look for toys" */
  scanButton: string;
  /** Shown on that button while scanning. Default: "Scanning" */
  scanning: string;
  /** Main panel heading when no toys are online. Default: "No toys online yet" */
  noToysTitle: string;
  /** Hint under that heading. Default: Press "<scanButton>" to find some */
  noToysHint: string;
};

/** What a theme file writes. Only `id`, `name` and `colors` are required. */
export type ThemeDefinition = {
  /** Unique, stable id (saved in settings), e.g. "midnight" */
  id: string;
  /** Name shown in the theme picker */
  name: string;
  /** "dark" (default) or "light": tells the webview how to draw native controls */
  scheme?: "dark" | "light";
  colors: ThemeColors;
  logo?: {
    /**
     * Gradient map for VibeCheck_logo.png. Its colours sit in brightness
     * bands: outlines ~0.03-0.09, purple "VIBE" ~0.26, coral lips and
     * "check" ~0.53-0.63, white highlights ~0.97.
     * Default: outlines -> text, "VIBE" -> secondary, lips -> primary.
     */
    stops?: LogoStop[];
    /** Glow behind the logo as hex. Default: primary. Set to null for none. */
    glow?: string | null;
  };
  /** Optional wording; anything left out uses the defaults */
  text?: Partial<ThemeText>;
  decorations?: {
    /** Little ears on top of the main panels. Default: false */
    ears?: boolean;
    /** Faint background pattern. Default: "none" */
    pattern?: PatternName;
    /**
     * Icon used for the cute accents (toy list, switch knobs, loading,
     * headings...). Any lucide-react icon. Default: Sparkles
     */
    icon?: LucideIcon;
    /** "round" (default) or "sharp" (square corners everywhere) */
    corners?: CornerStyle;
    /**
     * Shadow look for panels, buttons, tooltips and popups:
     * "glow" (default, soft), "neon" (bright glows), "hard" (solid offset
     * shadows) or "none"
     */
    shadows?: ShadowStyle;
    /** "rounded" (default), "mono" or "sans" */
    font?: FontStyle;
  };
};

/** A theme with every default filled in (what the app uses) */
export type Theme = {
  id: string;
  name: string;
  scheme: "dark" | "light";
  colors: Required<ThemeColors>;
  logo: {
    stops: LogoStop[];
    glow: string | null;
  };
  text: ThemeText;
  decorations: {
    ears: boolean;
    pattern: PatternName;
    icon: LucideIcon;
    corners: CornerStyle;
    shadows: ShadowStyle;
    font: FontStyle;
  };
};
