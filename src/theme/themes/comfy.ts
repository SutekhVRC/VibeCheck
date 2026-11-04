import { Coffee } from "lucide-react";
import { defineTheme } from "../defineTheme";

// Warm charcoal with soft apricot and oat accents.
// Cosy and neutral, with no ears or background pattern.
export default defineTheme({
  id: "comfy",
  name: "Comfy",
  colors: {
    background: "#1a1816",
    surface: "#23201d",
    raised: "#2d2925",
    border: "#443d36",
    text: "#f3ece4",
    muted: "#a39a8f",
    primary: "#e8a87c",
    secondary: "#c9b79c",
    success: "#a8c69f",
    danger: "#e48f8f",
    info: "#9cc1d6",
    warning: "#e8c27a",
  },
  text: {
    scanButton: "Look for toys",
    scanning: "Looking",
    noToysTitle: "Nothing here yet",
    noToysHint: "Press \u201cLook for toys\u201d and make yourself comfy",
  },
  decorations: {
    icon: Coffee,
  },
});
