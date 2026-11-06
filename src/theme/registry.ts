import type { Theme } from "./types";

/** Theme used on first launch, or if the saved theme no longer exists */
export const DEFAULT_THEME_ID = "vibecheck";

/*
 * Every file in ./themes/ is picked up automatically (Vite import.meta.glob),
 * so adding a theme is just adding a file there that default-exports
 * defineTheme({...}).
 */
const modules = import.meta.glob<{ default: Theme }>("./themes/*.ts", {
  eager: true,
});

function loadThemes(): Theme[] {
  const byId = new Map<string, Theme>();
  for (const [path, mod] of Object.entries(modules)) {
    const theme = mod.default;
    if (!theme?.id) {
      console.warn(
        `Theme file ${path} has no default export from defineTheme()`,
      );
      continue;
    }
    if (byId.has(theme.id)) {
      console.warn(`Duplicate theme id "${theme.id}" in ${path}; skipping it`);
      continue;
    }
    byId.set(theme.id, theme);
  }
  // Default theme first, the rest alphabetically
  return [...byId.values()].sort((a, b) =>
    a.id === DEFAULT_THEME_ID
      ? -1
      : b.id === DEFAULT_THEME_ID
        ? 1
        : a.name.localeCompare(b.name),
  );
}

export const themes: Theme[] = loadThemes();

export function getTheme(id: string | null | undefined): Theme {
  return (
    themes.find((t) => t.id === id) ??
    themes.find((t) => t.id === DEFAULT_THEME_ID) ??
    themes[0]
  );
}
