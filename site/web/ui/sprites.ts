// Pixel sprites: 9×9 bitmaps drawn as crisp SVG, centred horizontally in their cell. '#' is the main colour, 'o' the secondary colour (usually white), '.' empty.
// TONE sprites carry status: each tone has its own silhouette so status reads without colour.
// SHAPE sprites name a loop's result shape (vocabulary.toml [[shape]]). MARK sprites are small symbols used by components.
// Adding a tone or a shape means adding a bitmap here; see DESIGN.md, "Sprites".

import { s } from "./dom";

export type Tone = "good" | "warning" | "serious" | "critical" | "neutral";
export const TONES: Tone[] = ["good", "warning", "serious", "critical", "neutral"];

const TONE: Record<Tone, string[]> = {
  // disc with a tick
  good: ["..#####..", ".#######.", "#######o#", "######o##", "#o###o###", "##o#o####", "###o#####", ".#######.", "..#####.."],
  // triangle with "!"
  warning: ["....#....", "...###...", "...#o#...", "..##o##..", "..##o##..", ".###o###.", ".#######.", "####o####", "#########"],
  // diamond with "!"
  serious: ["....#....", "...#o#...", "..##o##..", ".###o###.", "####o####", ".#######.", "..##o##..", "...###...", "....#...."],
  // disc with a cross
  critical: ["..#####..", ".#######.", "##o###o##", "###o#o###", "####o####", "###o#o###", "##o###o##", ".#######.", "..#####.."],
  // hollow ring with "?"
  neutral: ["..#####..", ".#.....#.", "#..###..#", "#.#...#.#", "#....#..#", "#...#...#", "#.......#", ".#..#..#.", "..#####.."],
};

const SHAPE: Record<string, string[]> = {
  coverage: ["##.##.##.", "##.##.##.", ".........", "##.##.##.", "##.##.##.", ".........", "##.##.##.", "##.##.##.", "........."],
  discovery: [".####....", "#....#...", "#....#...", "#....#...", "#....#...", ".####....", ".....##..", "......##.", ".......##"],
  ladder: ["#.....#..", "#######..", "#.....#..", "#.....#..", "#######..", "#.....#..", "#.....#..", "#######..", "#.....#.."],
  scalar: ["....#....", "....#....", "....#....", "....#....", "#...#...#", ".#..#..#.", "..#.#.#..", "...###...", "....#...."],
  pareto: ["#........", "#........", "###......", "..#......", "..####...", ".....#...", ".....####", "........#", "........."],
  "reference-ratio": ["....#....", "...###...", "....#....", ".........", "#########", ".........", "....#....", "...###...", "....#...."],
  bracket: ["###...###", "#.......#", "#.......#", "#.......#", "#.......#", "#.......#", "#.......#", "#.......#", "###...###"],
  "open-to-bound": ["........#", ".....#..#", "......#.#", "#########", "......#.#", ".....#..#", "........#", ".........", "........."],
};

const MARK: Record<string, string[]> = {
  star: ["....#....", "....#....", "...###...", "#########", ".#######.", "..#####..", "..##.##..", ".##...##.", "#.......#"],
  square: ["#########", "#########", "#########", "#########", "#########", "#########", "#########", "#########", "#########"],
};

/** The CSS colour token of a tone. */
export const toneColor = (tone: Tone) => `var(--tone-${tone})`;

export function isTone(value: unknown): value is Tone {
  return typeof value === "string" && (TONES as string[]).includes(value);
}

/** How far to shift a bitmap right so its drawn pixels sit centred in the 9-wide cell (0.5 = one screen pixel at px 2). */
function centreOffset(bitmap: string[]): number {
  const cols = bitmap.flatMap((row) => [...row].flatMap((ch, x) => (ch === "." ? [] : [x])));
  return cols.length ? (8 - Math.max(...cols) - Math.min(...cols)) / 2 : 0;
}

function draw(bitmap: string[], color: string, px: number, alt: string, label?: string): SVGElement {
  const rects: SVGElement[] = [];
  const dx = centreOffset(bitmap); // bitmaps are drawn centred horizontally, whatever columns they leave empty
  bitmap.forEach((row, y) =>
    [...row].forEach((ch, x) => {
      if (ch === "#") rects.push(s("rect", { x: x + dx, y, width: 1, height: 1, fill: color }));
      else if (ch === "o") rects.push(s("rect", { x: x + dx, y, width: 1, height: 1, fill: alt }));
    }),
  );
  return s("svg", { class: "spr", width: 9 * px, height: 9 * px, viewBox: "0 0 9 9", role: label ? "img" : undefined, "aria-label": label, "aria-hidden": label ? undefined : "true" }, rects);
}

/** The sprite of a status tone, in that tone's colour. */
export const toneSprite = (tone: Tone, px = 2) => draw(TONE[tone], toneColor(tone), px, "var(--on-tone)");

/** The sprite of a loop shape; unknown shapes fall back to the neutral ring so a new shape is visible but obviously unfinished. */
export const shapeSprite = (shape: string, color = "currentColor", px = 2) => draw(SHAPE[shape] ?? TONE.neutral, color, px, "var(--win)", shape);
export const hasShapeSprite = (shape: string) => shape in SHAPE;

export const markSprite = (mark: keyof typeof MARK, color = "currentColor", px = 2) => draw(MARK[mark]!, color, px, "var(--win)");

/** Raw bitmaps, for chart markers that are drawn into a larger SVG. */
export const BITMAPS = { tone: TONE, shape: SHAPE, mark: MARK };
