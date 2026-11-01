// Charts drawn as crisp SVG on the screen. Two kinds exist; add a kind here rather than drawing SVG in a view.
//   recordChart:   one value per run over time, with the best-so-far as a step line and new records marked.
//   frontierChart: up to four series of (x, y) results where lower-left is better, with each series' frontier as a step line.
// Series colours come from --series-1..4 and every series also has its own marker shape. See DESIGN.md, "Charts".

import { h, s } from "./dom";
import { BITMAPS } from "./sprites";

const SERIES = ["var(--series-1)", "var(--series-2)", "var(--series-3)", "var(--series-4)"];
// marker shapes by series index: square, diamond, triangle, disc
const MARKERS = [BITMAPS.mark.square!, BITMAPS.tone.serious, BITMAPS.tone.warning, BITMAPS.tone.good];

function ticks(lo: number, hi: number, count = 4): number[] {
  const step0 = (hi - lo) / count;
  const mag = 10 ** Math.floor(Math.log10(step0));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((x) => x >= step0) ?? step0;
  const out: number[] = [];
  for (let v = Math.ceil(lo / step) * step; v <= hi + 1e-9; v += step) out.push(Number(v.toFixed(10)));
  return out;
}

// A bitmap marker. Filled draws every pixel; hollow draws only the edge pixels, with the inside in the window colour.
function marker(bitmap: string[], cx: number, cy: number, color: string, filled: boolean, size = 11) {
  const k = size / 9;
  const on = (x: number, y: number) => (bitmap[y]?.[x] ?? ".") !== ".";
  const edge = (x: number, y: number) => !on(x - 1, y) || !on(x + 1, y) || !on(x, y - 1) || !on(x, y + 1);
  return s(
    "g",
    { transform: `translate(${cx - size / 2},${cy - size / 2}) scale(${k})` },
    bitmap.flatMap((row, y) => [...row].map((ch, x) => (ch === "." ? null : s("rect", { x, y, width: 1, height: 1, fill: filled || edge(x, y) ? color : "var(--win)" })))),
  );
}

export interface RecordPoint {
  /** Time of the run, in ms since the epoch. */
  t: number;
  value: number;
}

export interface RecordChartOptions {
  points: RecordPoint[];
  better: "lower" | "higher";
  /** Formats a value for the axis and the record label, e.g. v => `${v}k`. */
  format?: (v: number) => string;
  /** One sentence for screen readers. */
  summary: string;
}

/** Each run as a hollow square, new records filled, the best so far as a step line. */
export function recordChart(o: RecordChartOptions): SVGElement {
  const fmt = o.format ?? String;
  const W = 560, H = 270, L = 66, R = 96, T = 36, B = 34;
  const t0 = o.points[0]!.t, t1 = o.points[o.points.length - 1]!.t;
  const vals = o.points.map((p) => p.value);
  const yt = ticks(Math.min(...vals), Math.max(...vals));
  const y0 = Math.min(yt[0]!, ...vals), y1 = Math.max(yt[yt.length - 1]!, ...vals);
  const X = (t: number) => L + ((t - t0) / (t1 - t0 || 1)) * (W - L - R);
  const Y = (v: number) => T + (1 - (v - y0) / (y1 - y0 || 1)) * (H - T - B);
  const beats = (a: number, b: number) => (o.better === "lower" ? a < b : a > b);
  let best = o.points[0]!.value;
  const runs = o.points.map((p, i) => {
    const isRecord = i === 0 || beats(p.value, best);
    if (isRecord) best = p.value;
    return { ...p, best, isRecord };
  });
  let path = `M${X(runs[0]!.t)},${Y(runs[0]!.best)}`;
  runs.slice(1).forEach((r) => (path += ` H${X(r.t)} V${Y(r.best)}`));
  const last = runs[runs.length - 1]!;
  const date = (t: number) => new Date(t).toLocaleDateString("en", { month: "short", day: "numeric", timeZone: "UTC" });
  const every = Math.max(1, Math.ceil(runs.length / 5));
  return s(
    "svg",
    { class: "chart", viewBox: `0 0 ${W} ${H}`, role: "img", "aria-label": o.summary },
    yt.map((v) => [s("line", { class: "grid", x1: L, x2: W - R, y1: Y(v), y2: Y(v) }), s("text", { x: L - 8, y: Y(v) + 5, "text-anchor": "end" }, fmt(v))]),
    runs.filter((_, i) => i % every === 0).map((r) => s("text", { x: X(r.t), y: H - 10, "text-anchor": "middle" }, date(r.t))),
    s("path", { class: "area", d: `${path} V${Y(o.better === "lower" ? y0 : y1)} H${X(t0)} Z`, fill: SERIES[0] }),
    s("path", { d: path, fill: "none", stroke: SERIES[0], "stroke-width": 3 }),
    runs.map((r) => marker(MARKERS[0]!, X(r.t), Y(r.value), r.isRecord ? SERIES[0]! : SERIES[1]!, r.isRecord, 9)),
    s("line", { class: "axis", x1: L, x2: W - R, y1: H - B, y2: H - B }),
    s("text", { class: "label", x: X(last.t) + 10, y: Y(last.best) - 4 }, "record"),
    s("text", { x: X(last.t) + 10, y: Y(last.best) + 14 }, fmt(last.best)),
    marker(MARKERS[0]!, L + 8, 10, SERIES[0]!, true, 9), s("text", { x: L + 18, y: 15 }, "new record"),
    marker(MARKERS[0]!, L + 134, 10, SERIES[1]!, false, 9), s("text", { x: L + 144, y: 15 }, "run that did not beat it"),
  );
}

export interface FrontierSeries {
  name: string;
  points: [number, number][];
}

export interface FrontierChartOptions {
  series: FrontierSeries[];
  xLabel: string;
  yLabel: string;
  summary: string;
}

/** Lower-left is better on both axes. Points on a series' frontier are filled, dominated points hollow. */
export function frontierChart(o: FrontierChartOptions): SVGElement {
  if (o.series.length > SERIES.length) throw new Error(`frontierChart supports at most ${SERIES.length} series`);
  const W = 900, H = 310, L = 56, R = 130, T = 26, B = 46;
  const xs = o.series.flatMap((x) => x.points.map((p) => p[0]));
  const ys = o.series.flatMap((x) => x.points.map((p) => p[1]));
  const xt = ticks(Math.min(...xs), Math.max(...xs), 6), yt = ticks(0, Math.max(...ys));
  const x0 = Math.min(xt[0]!, ...xs), x1 = Math.max(xt[xt.length - 1]!, ...xs), y0 = 0, y1 = Math.max(yt[yt.length - 1]!, ...ys);
  const X = (v: number) => L + ((v - x0) / (x1 - x0 || 1)) * (W - L - R);
  const Y = (v: number) => T + (1 - (v - y0) / (y1 - y0 || 1)) * (H - T - B);
  const fronts = o.series.map((x) => {
    const sorted = [...x.points].sort((a, b) => a[0] - b[0]);
    const front: [number, number][] = [];
    let bestY = Infinity;
    for (const p of sorted) if (p[1] < bestY) { front.push(p); bestY = p[1]; }
    return front;
  });
  return s(
    "svg",
    { class: "chart", viewBox: `0 0 ${W} ${H}`, role: "img", "aria-label": o.summary },
    yt.map((v) => [s("line", { class: "grid", x1: L, x2: W - R, y1: Y(v), y2: Y(v) }), s("text", { x: L - 8, y: Y(v) + 5, "text-anchor": "end" }, String(v))]),
    xt.map((v) => s("text", { x: X(v), y: H - 24, "text-anchor": "middle" }, String(v))),
    s("text", { x: (L + W - R) / 2, y: H - 4, "text-anchor": "middle" }, o.xLabel),
    s("text", { x: L, y: T - 10 }, o.yLabel),
    s("line", { class: "axis", x1: L, x2: W - R, y1: Y(0), y2: Y(0) }),
    o.series.map((series, i) => {
      const front = fronts[i]!;
      let d = `M${X(front[0]![0])},${Y(front[0]![1])}`;
      front.slice(1).forEach((p) => (d += ` H${X(p[0])} V${Y(p[1])}`));
      d += ` H${X(x1)}`;
      const lastY = front[front.length - 1]![1];
      return s(
        "g",
        {},
        s("path", { d, fill: "none", stroke: SERIES[i], "stroke-width": 3 }),
        series.points.map((p) => marker(MARKERS[i]!, X(p[0]), Y(p[1]), SERIES[i]!, front.includes(p))),
        s("text", { class: "label", x: X(x1) + 8, y: Y(lastY) + 5, style: `fill:${SERIES[i]}` }, series.name),
      );
    }),
    s("text", { x: W - R - 4, y: T - 10, "text-anchor": "end" }, "filled = on the frontier, hollow = dominated"),
  );
}

/** A legend for chart series when the series names are not written on the plot. */
export const seriesLegend = (names: string[]) => h("div", { class: "legend" }, names.map((n, i) => h("span", { class: "item" }, s("svg", { class: "spr", width: 14, height: 14, viewBox: "0 0 14 14", "aria-hidden": "true" }, marker(MARKERS[i]!, 7, 7, SERIES[i]!, true)), n)));
