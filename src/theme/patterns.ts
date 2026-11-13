/*
 * Background patterns a theme can pick with `decorations.pattern`.
 *
 * Each pattern is an SVG tile used as a CSS mask: only the shapes matter
 * (draw them in solid black; anything transparent is a gap), and the theme's
 * `secondary` colour shows through them very faintly. The tile repeats across
 * the whole window, so shapes are scattered at uneven positions, angles and
 * sizes to keep the repetition from being obvious.
 *
 * To add a pattern: add an entry to `patterns` (and an opacity below if it
 * needs to be stronger or fainter than the default).
 */

// -- Shapes, each drawn around (0, 0) at roughly 20-24px across --

const PAW = `<ellipse cx='0' cy='8' rx='9' ry='7'/><circle cx='-9' cy='-4' r='3.6'/><circle cx='-3' cy='-9' r='3.6'/><circle cx='4' cy='-9' r='3.6'/><circle cx='10' cy='-4' r='3.6'/>`;
const HEART = `<path d='M0 9C-5 5-11 1-11-4a5.5 5.5 0 0 1 11-2 5.5 5.5 0 0 1 11 2C11 1 5 5 0 9z'/>`;
const STAR = `<path d='M0-11 2.6-2.6 11 0 2.6 2.6 0 11-2.6 2.6-11 0-2.6-2.6z'/>`;
/** Kiss mark: upper lip with a cupid's bow, gap, lower lip */
const LIPS = `<path d='M-11 0C-8-3-5-6-2.5-5-1-4.5 0-3.5 0-3.5S1-4.5 2.5-5C5-6 8-3 11 0z'/><path d='M-11 1C-7 6.5 7 6.5 11 1 5 2.2-5 2.2-11 1z'/>`;
/** Leaf as two halves with a gap down the middle for the vein, plus a stem */
const LEAF = `<path d='M-0.7-11C-6.5-6-7 5-0.7 10.5z'/><path d='M0.7-11C6.5-6 7 5 0.7 10.5z'/><path d='M-0.6 10h1.2v5h-1.2z'/>`;
/** Crescent moon */
const MOON = `<path d='M3-10.5A11 11 0 1 0 11 3 8.5 8.5 0 0 1 3-10.5z'/>`;
const DOT = `<circle r='1'/>`;
/** Flame with a side tongue and a hollow inner flame (evenodd cuts it out) */
const FLAME = `<path fill-rule='evenodd' d='M0-13C1-8 6-7 8-2 9 1 9 4 7.5 6.5A8.5 7 0 0 1-7.5 6.5C-9 4-8.5 0-6-3-6-0.5-5 1-3.5 1.5-4.5-3-3-8 0-13zM0 9.5C-2.8 9.5-3.8 7.5-3.2 5.5-2.6 3.6-1 2.5 0 0 1 2.5 2.6 3.6 3.2 5.5 3.8 7.5 2.8 9.5 0 9.5z'/>`;
/** O-ring: a ring with the middle cut out (evenodd) */
const O_RING = `<path fill-rule='evenodd' d='M0-10A10 10 0 1 0 0 10 10 10 0 1 0 0-10zM0-6.8A6.8 6.8 0 1 1 0 6.8 6.8 6.8 0 1 1 0-6.8z'/>`;
/** Chain link: a long rounded loop */
const LINK = `<path fill-rule='evenodd' d='M-6-5A6 6 0 0 1 6-5V5A6 6 0 0 1-6 5zM-3.2-5A3.2 3.2 0 0 1 3.2-5V5A3.2 3.2 0 0 1-3.2 5z'/>`;
/** Chain seen side-on: two links face-on, joined by one edge-on through both */
const CHAIN = `<g transform='translate(0 -13) scale(0.8)'>${LINK}</g><rect x='-1.4' y='-9' width='2.8' height='18' rx='1.4'/><g transform='translate(0 13) scale(0.8)'>${LINK}</g>`;
/** Heart padlock: shackle on top, keyhole cut out of the heart */
const HEART_LOCK = `<path d='M-4.6-2.5V-7A4.6 4.6 0 0 1 4.6-7v4.5H2.3V-7A2.3 2.3 0 0 0-2.3-7v4.5z'/><path fill-rule='evenodd' d='M0 12C-5 8.5-10 5-10 0.5A4.8 4.8 0 0 1 0-1.3 4.8 4.8 0 0 1 10 0.5C10 5 5 8.5 0 12zM0 1.2A1.7 1.7 0 1 0 0 4.6 1.7 1.7 0 1 0 0 1.2zM-0.7 4H0.7V7.6H-0.7z'/>`;
/** Pitchfork: three pointed prongs on a crossbar, and a handle */
const PITCHFORK = `<path d='M-7-5v-7l1.5-3 1.5 3v7z'/><path d='M-1.5-5v-8l1.5-3 1.5 3v8z'/><path d='M4-5v-7l1.5-3 1.5 3v7z'/><path d='M-7-6h14v2.2h-14z'/><path d='M-1-4h2v16h-2z'/>`;

/** One placed copy of a shape: [shape, x, y, rotation in degrees, scale] */
type Placement = [string, number, number, number, number];

/** A square tile with shapes scattered at the given placements */
function scatter(size: number, placements: Placement[]) {
  const shapes = placements
    .map(
      ([shape, x, y, rotate, scale]) =>
        `<g transform='translate(${x} ${y}) rotate(${rotate}) scale(${scale})'>${shape}</g>`,
    )
    .join("");
  return `<svg xmlns='http://www.w3.org/2000/svg' width='${size}' height='${size}' viewBox='0 0 ${size} ${size}'><g fill='black'>${shapes}</g></svg>`;
}

/** Two copies of a shape, rotated, on a 160px tile (the original patterns) */
function pair(shape: string) {
  return scatter(160, [
    [shape, 28, 30, -20, 1],
    [shape, 112, 104, 18, 1],
  ]);
}

/** Thin square grid lines */
const GRID = `<svg xmlns='http://www.w3.org/2000/svg' width='40' height='40' viewBox='0 0 40 40'><path fill='black' d='M0 0h40v1H0zM0 0h1v40H0z'/></svg>`;

/** Knitted "V" stitches in columns, like a sweater */
const KNIT = `<svg xmlns='http://www.w3.org/2000/svg' width='12' height='11' viewBox='0 0 12 11'><g fill='black' transform='translate(6 5.5)'><ellipse rx='1.9' ry='4.6' transform='translate(-2.3 0) rotate(-32)'/><ellipse rx='1.9' ry='4.6' transform='translate(2.3 0) rotate(32)'/></g></svg>`;

/** Circuit traces with pads; lines leaving one edge re-enter on the other */
const CIRCUIT = `<svg xmlns='http://www.w3.org/2000/svg' width='160' height='160' viewBox='0 0 160 160'><g fill='none' stroke='black' stroke-width='1.4' stroke-linejoin='round'><path d='M0 30H50L70 50H112'/><path d='M160 30H136'/><path d='M30 0V18'/><path d='M30 160V140L50 120H70'/><path d='M160 96H118L100 114V136'/><path d='M0 96H24'/><circle cx='116' cy='50' r='3.5'/><circle cx='132' cy='30' r='3.5'/><circle cx='74' cy='120' r='3.5'/><circle cx='100' cy='140' r='3.5'/><circle cx='28' cy='96' r='3.5'/></g><g fill='black'><circle cx='30' cy='22' r='3'/><rect x='84' y='72' width='10' height='6' rx='1'/><rect x='140' y='128' width='6' height='10' rx='1'/></g></svg>`;

export const patterns = {
  none: null,

  // Simple patterns
  paws: pair(PAW),
  hearts: pair(HEART),
  stars: pair(STAR),
  grid: GRID,

  // VibeCheck: kiss marks with a few small hearts
  kisses: scatter(240, [
    [LIPS, 40, 46, -14, 1],
    [LIPS, 172, 32, 10, 0.8],
    [LIPS, 122, 130, -6, 1.1],
    [LIPS, 32, 192, 16, 0.85],
    [LIPS, 204, 184, -12, 0.95],
    [HEART, 92, 62, 12, 0.5],
    [HEART, 212, 106, -10, 0.45],
    [HEART, 70, 148, -18, 0.5],
    [HEART, 150, 216, 8, 0.45],
  ]),

  // Comfy: soft knitted texture
  knit: KNIT,

  // Cute: paw prints with little sparkles
  "paws-sparkles": scatter(220, [
    [PAW, 42, 46, -20, 0.9],
    [PAW, 160, 118, 18, 0.9],
    [PAW, 72, 182, -8, 0.75],
    [STAR, 132, 40, 0, 0.5],
    [STAR, 200, 192, 0, 0.4],
    [STAR, 22, 112, 0, 0.35],
    [STAR, 108, 100, 15, 0.3],
  ]),

  // Fox Den: autumn leaves with a couple of paw prints
  "leaves-paws": scatter(240, [
    [LEAF, 40, 40, -35, 1],
    [LEAF, 150, 72, 40, 0.85],
    [LEAF, 92, 162, 15, 1.1],
    [LEAF, 204, 192, -60, 0.9],
    [LEAF, 22, 204, 70, 0.8],
    [PAW, 194, 26, 25, 0.55],
    [PAW, 104, 100, -15, 0.5],
  ]),

  // Midnight: stars of different sizes, dots and a crescent moon
  starfield: scatter(260, [
    [STAR, 40, 52, 0, 0.6],
    [STAR, 182, 30, 15, 0.35],
    [STAR, 120, 142, 0, 0.8],
    [STAR, 228, 214, 10, 0.45],
    [STAR, 60, 212, 0, 0.35],
    [MOON, 210, 120, -20, 0.9],
    [DOT, 100, 40, 0, 1.6],
    [DOT, 150, 92, 0, 1.2],
    [DOT, 30, 130, 0, 1.4],
    [DOT, 90, 232, 0, 1.2],
    [DOT, 244, 62, 0, 1.1],
    [DOT, 168, 244, 0, 1.5],
    [DOT, 12, 18, 0, 1],
  ]),

  // Hardline: circuit traces
  circuit: CIRCUIT,

  // Neon Kitten: hearts with sparkles
  "hearts-sparkles": scatter(230, [
    [HEART, 40, 40, -15, 0.8],
    [HEART, 170, 92, 12, 0.65],
    [HEART, 90, 172, -8, 0.9],
    [HEART, 212, 210, 20, 0.55],
    [STAR, 132, 30, 0, 0.45],
    [STAR, 30, 122, 0, 0.35],
    [STAR, 200, 152, 0, 0.4],
    [STAR, 122, 110, 0, 0.3],
  ]),

  // Hellfire: flames and pitchforks, with hearts and sparks
  hellfire: scatter(240, [
    [FLAME, 38, 44, -10, 1],
    [FLAME, 174, 96, 14, 0.85],
    [FLAME, 70, 186, 4, 1.05],
    [PITCHFORK, 132, 34, 28, 0.9],
    [PITCHFORK, 206, 196, -24, 0.85],
    [HEART, 112, 120, -12, 0.55],
    [HEART, 24, 120, 16, 0.45],
    [STAR, 210, 30, 0, 0.4],
    [STAR, 150, 222, 0, 0.35],
    [STAR, 96, 82, 0, 0.3],
  ]),

  // Latex: O-rings, chain links and heart padlocks, with glints of shine
  latex: scatter(240, [
    [O_RING, 40, 42, 0, 0.9],
    [CHAIN, 166, 64, 32, 0.75],
    [HEART_LOCK, 108, 136, -10, 1],
    [O_RING, 214, 166, 0, 0.65],
    [CHAIN, 44, 196, -28, 0.7],
    [HEART_LOCK, 186, 222, 14, 0.7],
    [STAR, 106, 40, 0, 0.45],
    [STAR, 30, 120, 0, 0.35],
    [STAR, 222, 104, 0, 0.3],
    [STAR, 132, 214, 0, 0.35],
  ]),
} as const;

export type PatternName = keyof typeof patterns;

/**
 * How strongly each pattern shows (default 0.07). Thin lines need more;
 * dense textures need less so they stay in the background.
 */
const patternOpacity: Partial<Record<PatternName, number>> = {
  grid: 0.11,
  circuit: 0.1,
  knit: 0.04,
  // Small or dark-on-dark shapes need a little more to be noticed at all
  kisses: 0.085,
  starfield: 0.09,
  hellfire: 0.08,
  latex: 0.08,
};

export function patternStrength(name: PatternName) {
  return patternOpacity[name] ?? 0.07;
}

/** CSS url() for a pattern, or "none" */
export function patternUrl(name: PatternName) {
  const svg = patterns[name];
  return svg ? `url("data:image/svg+xml,${encodeURIComponent(svg)}")` : "none";
}
