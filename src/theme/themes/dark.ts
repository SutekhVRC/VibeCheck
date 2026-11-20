import { Circle } from "lucide-react";
import { defineTheme } from "../defineTheme";

// Flat and dark: near-black charcoal greys with crisp text and one muted
// pink accent. Primary and secondary are the same colour, so
// selections and buttons are solid instead of a two-colour gradient. No
// glows, no pattern, no ears.
export default defineTheme({
  id: "dark",
  name: "Dark",
  colors: {
    background: "#0c0c0e",
    surface: "#141416",
    raised: "#1d1d20",
    border: "#323237",
    text: "#f4f4f5",
    muted: "#a1a1aa",
    primary: "#d99ab5",
    secondary: "#d99ab5",
    success: "#8cc79a",
    danger: "#e08585",
    info: "#e6b3c9",
    warning: "#e0bd85",
  },
  logo: {
    // The logo in its own colours (as in vibecheck.ts), with the black
    // outlines made light so they show on the dark background; no glow
    stops: [
      { brightness: 0, color: "#f4f4f5" },
      { brightness: 0.1, color: "#f4f4f5" },
      { brightness: 0.18, color: "#7828a8" },
      { brightness: 0.29, color: "#7828a8" },
      { brightness: 0.36, color: "#c84a5a" },
      { brightness: 0.45, color: "#e85e66" },
      { brightness: 0.56, color: "#f87070" },
      { brightness: 0.68, color: "#f89090" },
      { brightness: 0.97, color: "#f8f8f8" },
      { brightness: 1, color: "#ffffff" },
    ],
    glow: null,
  },
  decorations: {
    icon: Circle,
    shadows: "none",
    font: "sans",
  },
});
