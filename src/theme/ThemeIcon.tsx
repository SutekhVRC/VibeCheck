import type { LucideProps } from "lucide-react";
import { useTheme } from "./ThemeProvider";

/** The active theme's accent icon (a paw for the Cute theme) */
export default function ThemeIcon(props: LucideProps) {
  const Icon = useTheme().theme.decorations.icon;
  return <Icon {...props} />;
}
