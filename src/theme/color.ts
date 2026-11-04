/** Small colour helpers for theme hex strings ("#rrggbb") */

export type Rgb = [number, number, number];

export function hexToRgb(hex: string): Rgb {
  const h = hex.replace("#", "");
  if (!/^[0-9a-fA-F]{6}$/.test(h)) {
    throw new Error(`Theme colour "${hex}" must be a 6-digit hex like #f7b2d9`);
  }
  return [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16)) as Rgb;
}

function toHex([r, g, b]: Rgb) {
  return `#${[r, g, b]
    .map((c) =>
      Math.round(Math.min(255, Math.max(0, c)))
        .toString(16)
        .padStart(2, "0"),
    )
    .join("")}`;
}

/** Blend from `a` (t = 0) to `b` (t = 1) */
export function mix(a: string, b: string, t: number) {
  const ca = hexToRgb(a);
  const cb = hexToRgb(b);
  return toHex(ca.map((c, i) => c + (cb[i] - c) * t) as Rgb);
}

/** CSS rgba() string, e.g. for glows */
export function withAlpha(hex: string, alpha: number) {
  const [r, g, b] = hexToRgb(hex);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

/** "r g b" channel string used by the --theme-* CSS variables */
export function toChannels(hex: string) {
  return hexToRgb(hex).join(" ");
}
