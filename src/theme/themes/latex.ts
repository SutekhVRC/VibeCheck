import { LockKeyhole } from "lucide-react";
import { defineTheme } from "../defineTheme";

// Glossy and kinky: shiny black with latex red and chrome-silver shine,
// O-rings, chains and heart padlocks
export default defineTheme({
  id: "latex",
  name: "Latex",
  colors: {
    background: "#060608",
    surface: "#101015",
    raised: "#1a1a21",
    border: "#2f2f3b",
    text: "#f6f4f8",
    muted: "#8f8c9c",
    primary: "#e8143f",
    secondary: "#d8dbe8",
    success: "#5fe3a1",
    danger: "#ff7a45",
    info: "#8fb8ff",
    warning: "#ffd25a",
    onAccent: "#0b0b10",
  },
  text: {
    scanButton: "Find playthings",
    scanning: "Searching",
    noToysTitle: "No playthings yet",
    noToysHint: "Press “Find playthings” to bring some in",
  },
  decorations: {
    pattern: "latex",
    icon: LockKeyhole,
    shadows: "gloss",
  },
});
