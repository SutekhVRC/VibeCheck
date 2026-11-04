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
    ears: true,          // ears on top of the main panels
    pattern: "hearts",   // "none" | "paws" | "hearts" | "stars" | "grid"
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
`registry.ts` (currently `velvet-kiss`).

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
| `decorations.ears` | `false` | Ears on the main panels, inner ear in `primary` |
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
takes `secondary` and the lips and "check" take `primary`. To customise it:

```ts
logo: {
  stops: [
    { brightness: 0, color: "#ffffff" },    // outlines (~0.03-0.09)
    { brightness: 0.12, color: "#ffffff" },
    { brightness: 0.26, color: "#d9c2ff" }, // "VIBE" (~0.26)
    { brightness: 0.4, color: "#d9c2ff" },
    { brightness: 0.55, color: "#ffc2e0" }, // lips, "check" (~0.53-0.63)
    { brightness: 0.75, color: "#ffc2e0" },
    { brightness: 1, color: "#ffffff" },    // highlights (~0.97)
  ],
},
```

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
| `velvet-kiss.ts` | **The default.** Colours taken from the logo, logo kept in its own colours (custom stops), neon glows, hearts |
| `comfy.ts` | Warm and neutral, only colours and an icon (the simplest template) |
| `cute.ts` | Pastel colours, ears, paw pattern, paw icon |
| `midnight.ts` | Minimal: no ears, faint stars |
| `fox-den.ts` | Warm palette with ears and paws |
| `hardline.ts` | Brutalist: sharp corners, hard shadows, mono font, grid, custom logo stops, no logo glow |
| `neon-kitten.ts` | Neon glows, hearts pattern, heart icon, ears |

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
