import { Star } from "lucide-react";
import { defineTheme } from "../defineTheme";

// Calm deep blue with teal accents, faint stars, no ears
export default defineTheme({
  id: "midnight",
  name: "Midnight",
  colors: {
    background: "#0d1117",
    surface: "#151b26",
    raised: "#1d2534",
    border: "#2c3a50",
    text: "#e6edf7",
    muted: "#8593a8",
    primary: "#5ed3c6",
    secondary: "#7aa2f7",
    success: "#7ee0a1",
    danger: "#f7768e",
    info: "#7dcfff",
    warning: "#e0af68",
  },
  text: {
    scanButton: "Scan for toys",
    scanning: "Scanning",
    noToysTitle: "No toys online",
    noToysHint: "Press \u201cScan for toys\u201d to search nearby",
  },
  decorations: {
    pattern: "stars",
    icon: Star,
  },
});
