import { hexToRgb, withAlpha } from "@/theme/color";
import { useTheme } from "@/theme/ThemeProvider";
import type { LogoStop } from "@/theme/types";
import { useId, useMemo } from "react";

/** Number of points in the filter's lookup table (blended in between) */
const TABLE_SIZE = 21;

/** Linear interpolation of one colour channel (0-1) across the stops at `t` */
function channelAt(stops: LogoStop[], t: number, channel: 0 | 1 | 2) {
  const value = (stop: LogoStop) => hexToRgb(stop.color)[channel] / 255;
  if (t <= stops[0].brightness) return value(stops[0]);
  for (let i = 1; i < stops.length; i++) {
    const prev = stops[i - 1];
    const next = stops[i];
    if (t <= next.brightness) {
      const span = next.brightness - prev.brightness || 1;
      const f = (t - prev.brightness) / span;
      return value(prev) + (value(next) - value(prev)) * f;
    }
  }
  return value(stops[stops.length - 1]);
}

/** Space-separated lookup table for one channel, for feFuncR/G/B */
function tableFor(stops: LogoStop[], channel: 0 | 1 | 2) {
  return Array.from({ length: TABLE_SIZE }, (_, i) =>
    channelAt(stops, i / (TABLE_SIZE - 1), channel).toFixed(4),
  ).join(" ");
}

/*
 * Shows an image recoloured with the active theme's logo gradient map,
 * using an SVG filter:
 * 1. feColorMatrix turns each pixel into its brightness (alpha untouched).
 * 2. feComponentTransfer looks that brightness up in a per-channel table
 *    built from the theme's logo stops.
 * The original image file is never modified.
 */
export default function ThemedLogo({
  src,
  className,
}: {
  src: string;
  className?: string;
}) {
  const { stops, glow } = useTheme().theme.logo;
  // useId contains ":" which is awkward inside url(#...), so strip it
  const filterId = `themed-logo-${useId().replace(/:/g, "")}`;
  const tables = useMemo(
    () => ({
      r: tableFor(stops, 0),
      g: tableFor(stops, 1),
      b: tableFor(stops, 2),
    }),
    [stops],
  );

  const filter = [
    `url(#${filterId})`,
    glow ? `drop-shadow(0 0 12px ${withAlpha(glow, 0.45)})` : "",
  ].join(" ");

  return (
    <>
      <svg
        aria-hidden
        width="0"
        height="0"
        style={{ position: "absolute", pointerEvents: "none" }}
      >
        <filter id={filterId} colorInterpolationFilters="sRGB">
          {/* Rec. 709 brightness into R, G and B; keep alpha */}
          <feColorMatrix
            type="matrix"
            values={[
              "0.2126 0.7152 0.0722 0 0",
              "0.2126 0.7152 0.0722 0 0",
              "0.2126 0.7152 0.0722 0 0",
              "0 0 0 1 0",
            ].join(" ")}
          />
          <feComponentTransfer>
            <feFuncR type="table" tableValues={tables.r} />
            <feFuncG type="table" tableValues={tables.g} />
            <feFuncB type="table" tableValues={tables.b} />
          </feComponentTransfer>
        </filter>
      </svg>
      <img src={src} className={className} style={{ filter }} />
    </>
  );
}
