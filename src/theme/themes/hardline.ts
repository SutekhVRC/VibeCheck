import { Cpu } from "lucide-react";
import { defineTheme } from "../defineTheme";

// Brutalist cyberpunk: raw black and concrete greys, hazard yellow and red,
// square corners, hard offset shadows, monospace type and a grid backdrop
export default defineTheme({
  id: "hardline",
  name: "Hardline",
  colors: {
    background: "#0a0a0a",
    surface: "#131313",
    raised: "#1c1c1c",
    border: "#3d3d3d",
    text: "#f0f0f0",
    muted: "#8c8c8c",
    primary: "#f5e400",
    secondary: "#ff2d2d",
    success: "#39ff14",
    danger: "#ff2d2d",
    info: "#00e5ff",
    warning: "#ff9500",
  },
  logo: {
    stops: [
      // Outlines -> off-white
      { brightness: 0, color: "#f0f0f0" },
      { brightness: 0.12, color: "#f0f0f0" },
      // "VIBE" -> hazard yellow
      { brightness: 0.26, color: "#f5e400" },
      { brightness: 0.4, color: "#f5e400" },
      // Lips and "check" -> red
      { brightness: 0.55, color: "#ff2d2d" },
      { brightness: 0.75, color: "#ff2d2d" },
      { brightness: 1, color: "#ffffff" },
    ],
    glow: null,
  },
  text: {
    scanButton: "SCAN",
    scanning: "SCANNING",
    noToysTitle: "NO DEVICES ONLINE",
    noToysHint: "Press SCAN to search for devices",
  },
  decorations: {
    pattern: "grid",
    icon: Cpu,
    corners: "sharp",
    shadows: "hard",
    font: "mono",
  },
});
