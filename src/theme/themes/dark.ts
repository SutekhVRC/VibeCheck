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
    glow: null,
  },
  decorations: {
    icon: Circle,
    shadows: "none",
    font: "sans",
  },
});
