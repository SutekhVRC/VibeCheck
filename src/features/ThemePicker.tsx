import { cn } from "@/lib/utils";
import { useTheme } from "@/theme/ThemeProvider";
import type { Theme } from "@/theme/types";

/** Small preview card drawn in the theme's own colours */
function ThemePreview({ theme }: { theme: Theme }) {
  const c = theme.colors;
  const Icon = theme.decorations.icon;
  return (
    <div
      className="flex h-14 items-center gap-2 rounded-xl border px-2"
      style={{ background: c.background, borderColor: c.border }}
    >
      <div
        className="flex h-9 flex-1 items-center gap-1.5 rounded-lg px-2"
        style={{ background: c.surface }}
      >
        <Icon className="h-4 w-4 shrink-0" style={{ color: c.primary }} />
        {[c.primary, c.secondary, c.success, c.danger].map((color, i) => (
          <span
            key={i}
            className="h-3 w-3 shrink-0 rounded-full"
            style={{ background: color }}
          />
        ))}
      </div>
    </div>
  );
}

/** Theme chooser for the Config screen. Applies (and saves) instantly. */
export default function ThemePicker() {
  const { theme: active, themes, setThemeId } = useTheme();
  return (
    <div className="grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-3">
      {themes.map((theme) => {
        const selected = theme.id === active.id;
        return (
          <button
            key={theme.id}
            type="button"
            onClick={() => setThemeId(theme.id)}
            aria-pressed={selected}
            className={cn(
              "flex flex-col gap-2 rounded-2xl border-2 p-2 text-left text-sm transition-all duration-200 hover:-translate-y-0.5",
              selected
                ? "border-theme-primary bg-theme-primary/10"
                : "border-transparent bg-theme-background/40 hover:border-theme-border",
            )}
          >
            <ThemePreview theme={theme} />
            <span
              className={cn(
                "px-1",
                selected
                  ? "font-semibold text-theme-primary"
                  : "text-theme-text",
              )}
            >
              {theme.name}
            </span>
          </button>
        );
      })}
    </div>
  );
}
