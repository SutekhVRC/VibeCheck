import { useTheme } from "./ThemeProvider";

/*
 * Ears drawn as inline SVG on top of a panel, for decorations.ears = "kitten",
 * "fox" or "horns". Place inside an element with the `theme-ears` class, which gives
 * it its own stacking context so the ears sit above the panel's background
 * and border but below everything inside the panel.
 *
 * The ear body uses the panel's own fill and border colours, so it reads as
 * the panel's edge rising into an ear. Only the inner ear (and the fox's tip)
 * is tinted. Colours come from the theme's CSS variables, so any theme can
 * use either style.
 *
 * The older "cat" ears are drawn in CSS (index.css, data-ears="cat").
 */

type EarShape = {
  /** Filled ear: base at y = 18, continuing to y = 22 to cover the border */
  body: string;
  /** Outer edge, running slightly past the base into the panel border */
  outline: string;
  /** Inner ear edge (closed with Z for the fill) */
  inner: string;
  /** Optional darker tip */
  tip?: string;
  /** Tip colour. Default: border */
  tipFill?: string;
  /** Draw the right-hand one mirrored (for shapes that lean to one side) */
  mirror?: boolean;
  /** Inner ear fill and edge colours */
  innerFill: string;
  innerStroke: string;
};

const shapes: Record<"kitten" | "fox" | "horns", EarShape> = {
  // Soft, curved sides
  kitten: {
    body: "M1 18 C4 10 9 3.5 12 1.2 Q13 0.5 14 1.2 C17 3.5 22 10 25 18 L25 22 L1 22 Z",
    outline:
      "M0.6 19.2 L1 18 C4 10 9 3.5 12 1.2 Q13 0.5 14 1.2 C17 3.5 22 10 25 18 L25.4 19.2",
    inner: "M6.5 18 C8.5 11.5 10.8 6.5 13 4.4 C15.2 6.5 17.5 11.5 19.5 18",
    innerFill: "rgb(var(--theme-primary) / 0.3)",
    innerStroke: "rgb(var(--theme-primary) / 0.55)",
  },
  // Straight sides, sharp tip, darker tip
  fox: {
    body: "M1 18 L12.2 1.4 Q13 0.4 13.8 1.4 L25 18 L25 22 L1 22 Z",
    outline: "M0.2 19.2 L1 18 L12.2 1.4 Q13 0.4 13.8 1.4 L25 18 L25.8 19.2",
    inner: "M6.5 18 L12.4 5.2 Q13 4.2 13.6 5.2 L19.5 18",
    tip: "M10.44 4 L12.2 1.4 Q13 0.4 13.8 1.4 L15.56 4 Z",
    innerFill: "rgb(var(--theme-secondary) / 0.28)",
    innerStroke: "rgb(var(--theme-primary) / 0.45)",
  },
  // Devil horns: wide base curving up and outward to a sharp tip, with a
  // tinted band along the inside curve
  horns: {
    body: "M8 18 C8 12 7 7 4.5 1.2 C10.5 4.5 16.5 10 18 18 L18 22 L8 22 Z",
    outline: "M8 19.4 L8 18 C8 12 7 7 4.5 1.2 C10.5 4.5 16.5 10 18 18 L18 19.4",
    inner: "M11 18 C10.8 13 9.6 9 7 4.8 C10.8 7.6 13.7 12 15 18",
    tip: "M4.5 1.2 C6.3 2.2 7.8 3.2 9 4.3 L6.3 5.6 C5.8 4.1 5.2 2.6 4.5 1.2 Z",
    tipFill: "rgb(var(--theme-primary))",
    innerFill: "rgb(var(--theme-primary) / 0.32)",
    innerStroke: "rgb(var(--theme-secondary) / 0.55)",
    mirror: true,
  },
};

/** How far the ear's base sits below the top of its 26 x 22 box (px) */
const BASE_Y = 18;

function Ear({ shape, side }: { shape: EarShape; side: "left" | "right" }) {
  return (
    <svg
      aria-hidden
      width="26"
      height="22"
      viewBox="0 0 26 22"
      className="pointer-events-none absolute"
      style={{
        zIndex: -1,
        top: -BASE_Y,
        [side]: 30,
        // Tilt outward, pivoting on the middle of the base
        transformOrigin: `50% ${(BASE_Y / 22) * 100}%`,
        transform: `rotate(${side === "left" ? -6 : 6}deg)`,
      }}
    >
      <g
        transform={
          shape.mirror && side === "right"
            ? "translate(26 0) scale(-1 1)"
            : undefined
        }
      >
        <path d={shape.body} style={{ fill: "rgb(var(--theme-surface))" }} />
        <path d={`${shape.inner} Z`} style={{ fill: shape.innerFill }} />
        <path
          d={shape.inner}
          fill="none"
          strokeWidth={0.8}
          style={{ stroke: shape.innerStroke }}
        />
        {shape.tip && (
          <path
            d={shape.tip}
            style={{ fill: shape.tipFill ?? "rgb(var(--theme-border))" }}
          />
        )}
        <path
          d={shape.outline}
          fill="none"
          strokeWidth={1}
          strokeLinejoin="round"
          style={{ stroke: "rgb(var(--theme-border) / 0.7)" }}
        />
      </g>
    </svg>
  );
}

/** Renders the active theme's SVG ears, if it uses kitten, fox or horns */
export default function PanelEars() {
  const { ears } = useTheme().theme.decorations;
  if (ears !== "kitten" && ears !== "fox" && ears !== "horns") return null;
  const shape = shapes[ears];
  return (
    <>
      <Ear shape={shape} side="left" />
      <Ear shape={shape} side="right" />
    </>
  );
}
