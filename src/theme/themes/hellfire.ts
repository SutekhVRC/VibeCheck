import { Flame } from "lucide-react";
import { defineTheme } from "../defineTheme";

// Infernal and flirty: crimson and hot pink on near-black, gold embers,
// devil horns, flames and pitchforks
export default defineTheme({
  id: "hellfire",
  name: "Hellfire",
  colors: {
    background: "#120608",
    surface: "#1f0b10",
    raised: "#2d1118",
    border: "#5a1e2a",
    text: "#fff0f2",
    muted: "#b88a93",
    primary: "#ff2e4d",
    secondary: "#ff7eb6",
    success: "#9be37a",
    danger: "#ff8a3d",
    info: "#c7a6ff",
    warning: "#ffc94a",
    onAccent: "#1a0508",
  },
  text: {
    scanButton: "Summon toys",
    scanning: "Summoning",
    noToysTitle: "Nothing has answered the summons",
    noToysHint: "Press “Summon toys” to call some up",
  },
  decorations: {
    ears: "horns",
    pattern: "hellfire",
    icon: Flame,
    shadows: "neon",
  },
});
