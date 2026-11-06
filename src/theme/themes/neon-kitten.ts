import { Heart } from "lucide-react";
import { defineTheme } from "../defineTheme";

// Cutesy cyberpunk: hot pink and cyan neon on deep violet, with kitty ears,
// hearts and bright neon glows
export default defineTheme({
  id: "neon-kitten",
  name: "Neon Kitten",
  colors: {
    background: "#0e0818",
    surface: "#1a1030",
    raised: "#251842",
    border: "#4a2f7a",
    text: "#fff0fb",
    muted: "#a98fcf",
    primary: "#ff5fd2",
    secondary: "#5ff3ff",
    success: "#7dffc4",
    danger: "#ff5c8a",
    info: "#7fc8ff",
    warning: "#ffe27a",
  },
  text: {
    scanButton: "Find toys",
    scanning: "Pinging",
    noToysTitle: "No toys online yet!",
    noToysHint: "Press \u201cFind toys\u201d to ping some \u2661",
  },
  decorations: {
    ears: "kitten",
    pattern: "hearts-sparkles",
    icon: Heart,
    shadows: "neon",
  },
});
