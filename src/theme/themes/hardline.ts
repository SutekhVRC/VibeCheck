import { Cpu } from "lucide-react";
import { defineTheme } from "../defineTheme";
import { logoGradient } from "../logo";

// Brutalist cyberpunk: raw black and concrete greys, hazard yellow and red,
// square corners, hard offset shadows, monospace type and a circuit-trace backdrop
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
    // Off-white outlines, hazard yellow "VIBE", red lips. Half shading keeps
    // the yellow bright instead of muddy.
    stops: logoGradient({
      outline: "#f0f0f0",
      vibe: "#f5e400",
      lips: "#ff2d2d",
      background: "#0a0a0a",
      shade: 0.5,
    }),
    glow: null,
  },
  text: {
    scanButton: "SCAN",
    scanning: "SCANNING",
    noToysTitle: "NO DEVICES ONLINE",
    noToysHint: "Press SCAN to search for devices",
  },
  decorations: {
    pattern: "circuit",
    icon: Cpu,
    corners: "sharp",
    shadows: "hard",
    font: "mono",
  },
});
