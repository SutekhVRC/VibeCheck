import { Circle } from "lucide-react";
import { defineTheme } from "../defineTheme";

// Flat and quiet: soft charcoal greys rather than black, muted lavender and
// sage accents, no glows, no pattern, no ears
export default defineTheme({
  id: "matte",
  name: "Matte",
  colors: {
    background: "#18181b",
    surface: "#1f1f23",
    raised: "#28282d",
    border: "#38383f",
    text: "#e4e4e7",
    muted: "#8e8e97",
    primary: "#b6a8d9",
    secondary: "#8fb5a9",
    success: "#93c49a",
    danger: "#d98c8c",
    info: "#93b4d9",
    warning: "#d9bb8c",
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
