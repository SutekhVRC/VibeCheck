import { Flame } from "lucide-react";
import { defineTheme } from "../defineTheme";

// The default theme, built from the VibeCheck logo: its coral lips and royal
// purple on velvety near-black wine, with sultry glows and faint kiss marks.
export default defineTheme({
  id: "vibecheck",
  name: "VibeCheck",
  colors: {
    background: "#140a12",
    surface: "#1f0f1c",
    raised: "#2a1426",
    border: "#4d2445",
    text: "#fdeef3",
    muted: "#b08aa0",
    // Logo coral, a touch hotter
    primary: "#f86878",
    // Logo purple (#7828a8), lightened so it reads on the dark background
    secondary: "#b25de0",
    success: "#8fdcb4",
    // Raspberry: distinct from the coral accent so "off" never looks selected
    danger: "#e0458a",
    info: "#a9c8ff",
    warning: "#ffc37a",
  },
  logo: {
    // Keep the logo's own colours; only the black outlines become pale pink
    // so they stay visible. Bands measured from VibeCheck_logo.png: purple
    // "VIBE" ~0.15-0.29 brightness, coral lips and "check" ~0.44-0.68.
    stops: [
      { brightness: 0, color: "#fdeef3" },
      { brightness: 0.1, color: "#fdeef3" },
      { brightness: 0.18, color: "#7828a8" },
      { brightness: 0.29, color: "#7828a8" },
      { brightness: 0.36, color: "#c84a5a" },
      { brightness: 0.45, color: "#e85e66" },
      { brightness: 0.56, color: "#f87070" },
      { brightness: 0.68, color: "#f89090" },
      { brightness: 0.97, color: "#f8f8f8" },
      { brightness: 1, color: "#ffffff" },
    ],
  },
  text: {
    scanButton: "Look for toys",
    scanning: "Searching",
    noToysTitle: "No toys online yet",
    noToysHint:
      "Press \u201cLook for toys\u201d to find something to play with",
  },
  decorations: {
    pattern: "kisses",
    icon: Flame,
    shadows: "neon",
  },
});
