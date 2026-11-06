import { mix } from "./color";
import type { LogoStop } from "./types";

/**
 * Builds a gradient map for VibeCheck_logo.png from three colours, keeping
 * the logo's original shading instead of flattening each part to one colour.
 *
 * The logo's colours sit in brightness bands (measured from the PNG):
 * - outlines ~0.03-0.09
 * - purple "VIBE" ~0.15-0.29
 * - coral lips and "check" ~0.44-0.68, with darker lip shadows from ~0.36
 * - white highlights ~0.97
 *
 * "VIBE" and the lip shadows are darkened towards the background so they
 * keep contrast against light outlines; lip highlights get a little lighter.
 * `shade` (0-1, default 1) scales how much darker the shaded parts get;
 * lower it for very saturated colours that turn muddy when darkened.
 */
export function logoGradient({
  outline,
  vibe,
  lips,
  background,
  shade = 1,
}: {
  outline: string;
  vibe: string;
  lips: string;
  background: string;
  shade?: number;
}): LogoStop[] {
  return [
    { brightness: 0, color: outline },
    { brightness: 0.1, color: outline },
    { brightness: 0.18, color: mix(vibe, background, 0.5 * shade) },
    { brightness: 0.29, color: mix(vibe, background, 0.32 * shade) },
    { brightness: 0.36, color: mix(lips, background, 0.45 * shade) },
    { brightness: 0.45, color: mix(lips, background, 0.22 * shade) },
    { brightness: 0.56, color: lips },
    { brightness: 0.68, color: mix(lips, "#ffffff", 0.25) },
    { brightness: 0.97, color: "#f8f8f8" },
    { brightness: 1, color: "#ffffff" },
  ];
}
