import { PawPrint } from "lucide-react";
import { defineTheme } from "../defineTheme";

// Pastel colours on dark plum, with ears and paw prints
export default defineTheme({
  id: "cute",
  name: "Cute",
  colors: {
    background: "#17131f",
    surface: "#221c2e",
    raised: "#2d2540",
    border: "#43395c",
    text: "#fbefff",
    muted: "#9d90b8",
    primary: "#f7b2d9",
    secondary: "#c7b3f5",
    success: "#9fe3c8",
    danger: "#f29bb0",
    info: "#a5d4f2",
    warning: "#ffc9a8",
  },
  text: {
    scanButton: "Look for toys",
    scanning: "Sniffing",
    noToysTitle: "No online toys... yet!",
    noToysHint: "Press \u201cLook for toys\u201d to sniff some out",
  },
  decorations: {
    ears: true,
    pattern: "paws",
    icon: PawPrint,
  },
});
