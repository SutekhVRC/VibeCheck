# VibeCheck themes

## Making a theme

1. Copy `themes/comfy.ts` (or any other theme) to a new file in `themes/`,
   e.g. `themes/bunny.ts`.
2. Change the `id` (must be unique) and `name`, and pick your colours.
3. That's it. Every file in `themes/` is picked up automatically and shows
   up in **Config → Theme**.

```ts
import { Rabbit } from "lucide-react";
import { defineTheme } from "../defineTheme";

export default defineTheme({
  id: "bunny",          // unique, saved in settings: don't change it later
  name: "Bunny",        // shown in the picker
  colors: {
    background: "#1b1720",
    surface: "#262030",
    raised: "#31293d",
    border: "#4a3f5c",
    text: "#fdf6ff",
    muted: "#a497b8",
    primary: "#ffc2e0",
    secondary: "#d9c2ff",
    success: "#b5ecc9",
    danger: "#ff9fb2",
    info: "#b3dcff",
    warning: "#ffd9b3",
  },
  // Everything below is optional
  decorations: {
    ears: "cat",         // "cat" | "kitten" | "fox" | "horns" | "none"
    pattern: "hearts",   // see patterns.ts, e.g. "paws", "kisses", "starfield"
    icon: Rabbit,        // any lucide-react icon
    corners: "round",    // "round" | "sharp"
    shadows: "glow",     // "glow" | "neon" | "hard" | "none"
    font: "rounded",     // "rounded" | "mono" | "sans"
  },
});
```

Colours must be 6-digit hex (`#rrggbb`). A typo throws an error naming the
bad value when the app starts.

The default theme for new users is set by `DEFAULT_THEME_ID` in
`registry.ts` (currently `vibecheck`).

## Colour roles

Components never use raw colours, only these roles. Pick a colour for each.

| Role | Used for |
|---|---|
| `background` | Page background, input boxes |
| `surface` | Main panels (sidebar, settings pane), toasts |
| `raised` | Boxes inside panels, tooltips, toy buttons, tabs |
| `border` | Borders and dividers |
| `text` | Main text |
| `muted` | Secondary text, placeholders, offline toys |
| `primary` | Main accent: buttons, selection, switches, slider fill |
| `secondary` | Second accent: gradients with primary, glows, icons |
| `success` | App-enabled switch, update dot, success toasts, full battery |
| `danger` | App-disabled switch, remove buttons, error toasts, empty battery |
| `info` | Links, info toasts |
| `warning` | Warning toasts |
| `onAccent` | *Optional.* Text on primary/secondary fills (button labels). Defaults to `background` |

Tip: for a **light theme**, set `scheme: "light"` and give `onAccent` a dark
colour. Keep in mind the GitHub and Discord logos in the sidebar are white,
so they need a darker `background` behind them to stay visible.

## Optional settings

| Setting | Default | What it does |
|---|---|---|
| `scheme` | `"dark"` | How the webview draws native controls (dropdown lists, scrollbars) |
| `decorations.ears` | `"none"` | `"cat"` (rounded, inner ear in `primary`), `"kitten"` (curved sides, `primary` inner ear) `"fox"` (straight sides, `secondary` inner ear, `border` tip) or `"horns"` (devil horns curving outward, `primary` band and tip, mirrored on the right). Kitten, fox and horns are drawn as SVG by `PanelEars.tsx`. `true` means `"cat"` |
| `decorations.pattern` | `"none"` | Faint background pattern in `secondary` |
| `decorations.icon` | `Sparkles` | Accent icon: toy list, switch knobs, loading, headings |
| `decorations.corners` | `"round"` | `"sharp"` squares off every corner (switches, pills, panels) |
| `decorations.shadows` | `"glow"` | `"glow"` soft accent glows, `"neon"` bright glows, `"hard"` solid offset shadows, `"none"` |
| `decorations.font` | `"rounded"` | `"rounded"`, `"mono"` or `"sans"` |
| `logo.stops` | from colours | Logo recolouring (see below) |
| `logo.glow` | `primary` | Glow behind the logo; `null` for none |
| `text` | neutral wording | Scan button and "no toys" message (see below) |

### Wording

Themes can change a few bits of text to match their personality. Leave any
out to use the default.

```ts
text: {
  scanButton: "Look for toys",   // sidebar scan button
  scanning: "Sniffing",          // that button while scanning
  noToysTitle: "No toys online yet",
  noToysHint: "Press \u201cLook for toys\u201d to find some",
},
```

If you only set `scanButton`, the default hint uses it automatically.

### Logo colours

The logo is recoloured with a gradient map: each pixel's brightness (0 = black,
1 = white) is swapped for a colour. By default outlines take `text`, "VIBE"
takes `secondary` and the lips and "check" take `primary`, with the logo's
shading kept: "VIBE" and the lip shadows are darkened towards `background`.

To pick different colours but keep the shading, use `logoGradient` (from
`logo.ts`):

```ts
import { logoGradient } from "../logo";

logo: {
  stops: logoGradient({
    outline: "#f0f0f0", // outlines
    vibe: "#f5e400",    // "VIBE"
    lips: "#ff2d2d",    // lips and "check"
    background: "#0a0a0a",
    shade: 0.5,         // optional: less darkening for very saturated colours
  }),
},
```

For full control, pass your own `stops` list. The logo's brightness bands are:
outlines ~0.03-0.09, "VIBE" ~0.15-0.29, lip shadows from ~0.36, lips and
"check" ~0.44-0.68, highlights ~0.97. See `vibecheck.ts` for an example that
keeps the logo's original colours.

### Adding a background pattern

Add an entry to `patterns.ts`. Patterns are SVG shapes used as a mask, so
draw them in any solid colour; the theme decides the actual colour. Thin
shapes can get a stronger opacity in `patternOpacity` there.

### Adding a shadow style or font

Shadow styles and font stacks live in `styles.ts`. Add an entry to
`shadowStyles` or `fontStacks` and its name becomes a valid option.

## Example themes

| Theme | Shows off |
|---|---|
| `vibecheck.ts` | **The default.** Colours taken from the logo, logo kept in its own colours (custom stops), neon glows, kisses pattern |
| `comfy.ts` | Warm and neutral: colours, an icon and a knit texture (the simplest template) |
| `cute.ts` | Pastel colours, cat ears, paws-and-sparkles pattern, paw icon |
| `midnight.ts` | Minimal: no ears, faint starfield |
| `fox-den.ts` | Warm palette with fox ears, leaves-and-paws pattern |
| `hardline.ts` | Brutalist: sharp corners, hard shadows, mono font, circuit pattern, custom logo stops, no logo glow |
| `neon-kitten.ts` | Neon glows, hearts-and-sparkles pattern, heart icon, kitten ears |
| `hellfire.ts` | Crimson and pink on near-black, devil horns, flames-and-pitchforks pattern, flame icon, custom wording and `onAccent` |

## How it works

- `ThemeProvider.tsx` writes the active theme to `<html>` as CSS variables
  (`--theme-primary: 247 178 217`, `--theme-shadow-panel`, `--theme-font`),
  plus `data-ears` / `data-pattern` / `data-corners`.
- Tailwind's `theme-*` colours read those variables, so classes like
  `bg-theme-primary/20` work everywhere (`tailwind.config.cjs`).
- `index.css` uses them directly: `rgb(var(--theme-primary) / 0.5)`.
- In components, `useTheme()` gives the active theme (for colours computed in
  code, like toasts and the battery icon), and `<ThemeIcon />` draws the
  theme's accent icon.
- The chosen theme id is saved in `localStorage` under `vibecheck-theme`.
