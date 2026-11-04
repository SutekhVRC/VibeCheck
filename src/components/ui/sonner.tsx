import { Bell, HeartCrack, PawPrint, Sparkles } from "lucide-react";
import type { CSSProperties } from "react";
import { Toaster as Sonner } from "sonner";

type ToasterProps = React.ComponentProps<typeof Sonner>;

/*
 * Sonner styles toasts through CSS variables on the toaster element.
 * Setting them inline overrides sonner's built-in light/dark palettes,
 * so every toast type picks up the pastel theme.
 */
const cuteToastVars = {
  "--normal-bg": "#221c2e",
  "--normal-border": "rgba(199, 179, 245, 0.45)",
  "--normal-text": "#fbefff",

  "--success-bg": "#1b2a26",
  "--success-border": "rgba(159, 227, 200, 0.55)",
  "--success-text": "#9fe3c8",

  "--info-bg": "#1b2434",
  "--info-border": "rgba(165, 212, 242, 0.55)",
  "--info-text": "#a5d4f2",

  "--warning-bg": "#2d2420",
  "--warning-border": "rgba(255, 201, 168, 0.55)",
  "--warning-text": "#ffc9a8",

  "--error-bg": "#2e1c27",
  "--error-border": "rgba(242, 155, 176, 0.6)",
  "--error-text": "#f7b8c6",

  "--border-radius": "1.25rem",
  // Sonner sets its own font stack; use the app's rounded font instead
  fontFamily: "inherit",
} as CSSProperties;

const Toaster = ({ ...props }: ToasterProps) => {
  return (
    <Sonner
      theme="dark"
      richColors
      style={cuteToastVars}
      icons={{
        success: <PawPrint className="h-4 w-4" />,
        info: <Sparkles className="h-4 w-4" />,
        warning: <Bell className="h-4 w-4" />,
        error: <HeartCrack className="h-4 w-4" />,
      }}
      toastOptions={{
        style: {
          // Error messages put backend details on a second line with "\n"
          whiteSpace: "pre-line",
          boxShadow: "0 10px 30px -10px rgba(199, 179, 245, 0.45)",
        },
      }}
      {...props}
    />
  );
};

export { Toaster };
