import {
  createContext,
  ReactNode,
  useContext,
  useLayoutEffect,
  useState,
} from "react";
import { toChannels } from "./color";
import { patternStrength, patternUrl } from "./patterns";
import { getTheme, themes } from "./registry";
import { fontStacks, shadowStyles } from "./styles";
import type { Theme } from "./types";

const STORAGE_KEY = "vibecheck-theme";

/** "onAccent" -> "on-accent" */
function kebab(s: string) {
  return s.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);
}

/*
 * Writes a theme onto <html>:
 * - every colour as a --theme-<role> CSS variable ("r g b"), which the
 *   Tailwind `theme-*` colours and index.css read
 * - data-ears / data-pattern / data-corners attributes and --theme-pattern
 *   for decorations
 * - --theme-shadow-* and --theme-font for the shadow and font styles
 * - color-scheme and the dark/light class for native controls and shadcn
 */
function applyTheme(theme: Theme) {
  const root = document.documentElement;
  for (const [role, hex] of Object.entries(theme.colors)) {
    root.style.setProperty(`--theme-${kebab(role)}`, toChannels(hex));
  }
  root.style.setProperty(
    "--theme-pattern",
    patternUrl(theme.decorations.pattern),
  );
  root.style.setProperty(
    "--theme-pattern-opacity",
    String(patternStrength(theme.decorations.pattern)),
  );
  root.dataset.ears = theme.decorations.ears;
  root.dataset.pattern = theme.decorations.pattern;
  root.dataset.corners = theme.decorations.corners;

  const shadows = shadowStyles[theme.decorations.shadows];
  for (const [name, value] of Object.entries(shadows)) {
    root.style.setProperty(`--theme-shadow-${name}`, value);
  }
  root.style.setProperty("--theme-font", fontStacks[theme.decorations.font]);
  root.style.colorScheme = theme.scheme;
  root.classList.toggle("dark", theme.scheme === "dark");
  root.classList.toggle("light", theme.scheme === "light");
}

type ThemeContextValue = {
  theme: Theme;
  themes: Theme[];
  setThemeId: (id: string) => void;
};

const ThemeContext = createContext<ThemeContextValue | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState(() =>
    getTheme(localStorage.getItem(STORAGE_KEY)),
  );

  // Layout effect: applied before the first paint, so no flash of no colours
  useLayoutEffect(() => applyTheme(theme), [theme]);

  function setThemeId(id: string) {
    const next = getTheme(id);
    localStorage.setItem(STORAGE_KEY, next.id);
    setTheme(next);
  }

  return (
    <ThemeContext.Provider value={{ theme, themes, setThemeId }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) throw new Error("useTheme must be used within ThemeProvider");
  return context;
}
