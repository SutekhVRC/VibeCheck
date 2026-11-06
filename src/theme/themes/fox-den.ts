import { PawPrint } from "lucide-react";
import { defineTheme } from "../defineTheme";

// Warm cosy browns with orange and cream, fox ears and paws
export default defineTheme({
  id: "fox-den",
  name: "Fox Den",
  colors: {
    background: "#1a1310",
    surface: "#261b16",
    raised: "#33251e",
    border: "#4e3a2e",
    text: "#fff3e6",
    muted: "#b39b88",
    primary: "#ffa96b",
    secondary: "#ffd8a8",
    success: "#b8e0a0",
    danger: "#f28b82",
    info: "#9fd3e0",
    warning: "#ffd166",
  },
  text: {
    scanButton: "Look for toys",
    scanning: "Sniffing",
    noToysTitle: "No toys in the den yet",
    noToysHint: "Press \u201cLook for toys\u201d to sniff some out",
  },
  decorations: {
    ears: "fox",
    pattern: "leaves-paws",
    icon: PawPrint,
  },
});
