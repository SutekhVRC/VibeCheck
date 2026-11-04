/*
 * Background patterns a theme can pick with `decorations.pattern`.
 *
 * Each pattern is an SVG tile used as a CSS mask: only the shapes matter
 * (draw them in any solid colour), and the theme's `secondary` colour shows
 * through them very faintly. To add a pattern, add an entry here.
 */

const PAW = `<ellipse cx='0' cy='8' rx='9' ry='7'/><circle cx='-9' cy='-4' r='3.6'/><circle cx='-3' cy='-9' r='3.6'/><circle cx='4' cy='-9' r='3.6'/><circle cx='10' cy='-4' r='3.6'/>`;
const HEART = `<path d='M0 9C-5 5-11 1-11-4a5.5 5.5 0 0 1 11-2 5.5 5.5 0 0 1 11 2C11 1 5 5 0 9z'/>`;
const STAR = `<path d='M0-11 2.6-2.6 11 0 2.6 2.6 0 11-2.6 2.6-11 0-2.6-2.6z'/>`;

/** Two copies of a shape, rotated, on a 160px tile */
function tile(shape: string) {
  return `<svg xmlns='http://www.w3.org/2000/svg' width='160' height='160' viewBox='0 0 160 160'><g fill='black'><g transform='translate(28 30) rotate(-20)'>${shape}</g><g transform='translate(112 104) rotate(18)'>${shape}</g></g></svg>`;
}

/** Thin square grid lines (a full-tile pattern, not two copies of a shape) */
const GRID = `<svg xmlns='http://www.w3.org/2000/svg' width='40' height='40' viewBox='0 0 40 40'><path fill='black' d='M0 0h40v1H0zM0 0h1v40H0z'/></svg>`;

export const patterns = {
  none: null,
  paws: tile(PAW),
  hearts: tile(HEART),
  stars: tile(STAR),
  grid: GRID,
} as const;

export type PatternName = keyof typeof patterns;

/** How strongly each pattern shows (default 0.05); thin lines need more */
const patternOpacity: Partial<Record<PatternName, number>> = {
  grid: 0.09,
};

export function patternStrength(name: PatternName) {
  return patternOpacity[name] ?? 0.05;
}

/** CSS url() for a pattern, or "none" */
export function patternUrl(name: PatternName) {
  const svg = patterns[name];
  return svg ? `url("data:image/svg+xml,${encodeURIComponent(svg)}")` : "none";
}
