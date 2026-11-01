// Style guide (#/guide): every token, sprite and component, drawn by the same code the views use.
// When you add or change a component, add or update its section here; this route is how agents and reviewers see the system.
// Sample values in this file are invented and labelled as such; never copy them into a view.

import type { Model } from "../../src/model";
import { h, type Child } from "../ui/dom";
import { chips, cols, counters, dialogue, empty, fields, gauge, list, page, small, split, table, tag, win } from "../ui/components";
import { frontierChart, recordChart } from "../ui/charts";
import { slotLegend, slotMatrix, miniGrid, type Grid, type Slot } from "../ui/matrix";
import { markSprite, shapeSprite, toneSprite, TONES, BITMAPS, type Tone } from "../ui/sprites";
import { status, statusOf } from "../ui/status";

const SAMPLE = "SAMPLE DATA";
const code = (text: string) => h("code", {}, text);
const usage = (builder: string, rule: Child) => h("p", { class: "small muted" }, code(builder), " — ", rule);

const swatch = (token: string, note = "") => h("div", { class: "swatch" }, h("i", { style: `background:var(${token})` }), h("span", {}, code(token), note ? small(note) : null));

function tokens() {
  return [
    win("Tokens: screen", [usage("styles/tokens.css", "views use these through components; a raw colour outside tokens.css is a bug."), h("div", { class: "swatches" }, ["--lcd", "--win", "--ink", "--ink-2", "--ink-3", "--rule", "--link", "--sel", "--sel-ink"].map((t) => swatch(t)))]),
    cols(
      win("Tokens: tones", [usage("--tone-*", "one per vocabulary tone; only ever drawn with that tone's sprite."), h("div", { class: "swatches" }, TONES.map((t) => swatch(`--tone-${t}`)))]),
      win("Tokens: series", [usage("--series-1..4", "categories that are not statuses (chart series); always paired with a marker shape."), h("div", { class: "swatches" }, [1, 2, 3, 4].map((i) => swatch(`--series-${i}`)))]),
    ),
    win("Tokens: case", [usage("--case-*, --bezel*, --button-*, --hinge-*, --desk*, --cart*", "the shell only. Never on the screen."), h("div", { class: "swatches" }, ["--case", "--case-print", "--bezel", "--stripe-1", "--stripe-2", "--button-ab", "--button-pad", "--led-on", "--hinge-mid", "--desk", "--cart-label-1", "--cart-stream"].map((t) => swatch(t)))]),
    win("Type", [
      usage("--text-*", "one face on the screen, four sizes. DotGothic16 is drawn on a 16px grid, so 16 and 32 are the crisp sizes."),
      h("div", { class: "type-ramp" },
        h("div", { class: "t-title" }, "Title 32 — page titles, counters"),
        h("div", { class: "t-heading" }, "Heading 20 — headings inside a window"),
        h("div", { class: "t-body" }, "Body 16 — everything else, including tables"),
        h("div", { class: "t-small" }, "Small 14 — secondary lines, captions, axis labels"),
        h("div", { class: "prose" }, "Prose 15 (system face) — only for rendered Markdown such as the README."),
        h("div", {}, code("code"), " — file names, revisions, evidence levels"),
      ),
    ]),
  ];
}

function sprites() {
  return cols(
    win("Sprites: tones", [usage("toneSprite(tone)", "each tone has its own silhouette; this is what makes status readable without colour."), chips(TONES.map((t) => status(t, t)))]),
    win("Sprites: shapes and marks", [
      usage("shapeSprite(id), markSprite(id)", "one per vocabulary shape; a shape with no bitmap falls back to the neutral ring."),
      chips(Object.keys(BITMAPS.shape).map((x) => h("span", { class: "status" }, shapeSprite(x), x))),
      chips([h("span", { class: "status" }, markSprite("star", "var(--series-4)"), "star: best value"), h("span", { class: "status" }, markSprite("square"), "square: chart marker")]),
    ]),
  );
}

function sampleSlots(seed: number): Slot[] {
  return Array.from({ length: 256 }, (_, i) => {
    const tone: Tone = i === 0x10 + seed ? "critical" : i % 53 === 7 ? "neutral" : i === 0xcb ? "neutral" : "good";
    const marked = seed === 0 && i === 0xcb;
    const key = i.toString(16).toUpperCase().padStart(2, "0");
    return { key, label: `${key}: ${tone}`, tone, marked, glyph: marked ? "CB" : tone === "critical" ? "✕" : undefined, detail: fields([["Sample", `slot ${key}`], ["Tone", status(tone, tone)]]) };
  });
}

function components(m: Model) {
  const grids: Grid[] = [{ id: "a", slots: sampleSlots(0) }, { id: "b", corner: "B", slots: sampleSlots(3) }];
  const day = 86400000;
  const runs = [1284, 1279, 1291, 1250, 1262, 1247, 1231, 1240, 1198, 1205, 1199, 1176, 1188, 1142, 1150, 1139, 1144, 1121].map((value, i) => ({ t: Date.UTC(2026, 9, 6) + i * 1.5 * day, value }));
  return [
    win("Page and window", [
      usage("page({ title, sub, crumb }, ...blocks)", "every route returns one page. Detail routes pass crumb to link back to their list."),
      usage("win(title, body, { aside | flag })", "every block of content sits in a window. Title: a short noun phrase. aside: a count or unit. flag: marks synthetic or provisional content."),
      usage("cols(...blocks)", "two or three related windows side by side; they stack on narrow screens."),
      cols(win("Plain window", "Body text.", { aside: "12 rows" }), win("Flagged window", "Content that is not real data yet.", { flag: SAMPLE })),
    ]),
    win("Dialogue", [
      usage("dialogue(body, tone?)", "a message about the data, not the data: a file is not adopted, a check found nothing. Give a tone when it reports a status."),
      dialogue("A message with no status."),
      dialogue("Not adopted yet: loops.toml (proposal).", "warning"),
    ]),
    win("Status, tag, chips", [
      usage("statusOf(terms, id)", "any vocabulary status: sprite + id, meaning on hover. status(tone, label) when the tone comes from code (severity, gate verdict)."),
      usage("tag(text, quiet?)", "a plain label with no status: a shape name, “baseline”, a maturity."),
      usage("chips(items)", "several statuses on one wrapping line, such as counts by status."),
      chips([statusOf(m.vocabulary.capability_status, "built"), statusOf(m.vocabulary.capability_status, "missing"), statusOf(m.vocabulary.ticket_status, "in-progress"), tag("baseline"), tag("quiet tag", true)]),
    ]),
    win("Table", [
      usage("table(columns, rows)", "rows are selectable: ↑↓ move the cursor, the info screen reads out the whole row, A opens the row's href. Give rows an id: it is the readout's title, and #/route/<id> selects the row. num: right-aligned numbers; nowrap: ids. On a main screen under 640px only the first two columns show, so put the id and the status first."),
      table(
        [{ label: "Status", nowrap: true }, { label: "Item", nowrap: true }, { label: "Description" }, { label: "Count", num: true }],
        [
          { id: "sample-1", cells: [status("good", "built"), "sample-1", "A row with a status, an id and a number.", 12] },
          { id: "sample-2", cells: [status("critical", "missing"), "sample-2", ["A row with a secondary line.", small("Secondary lines use small().")], 3] },
          { id: "sample-3", select: false, cells: [status("neutral", "unverified"), "sample-3", "A row that the cursor skips (select: false).", 0] },
        ],
      ),
    ]),
    cols(
      win("Fields", [usage("fields([[label, value]])", "label / value pairs for one record. undefined shows “not specified”, so gaps stay visible."), fields([["Objective", "A recorded value."], ["Candidates", undefined], ["Gates", list(["build", "replay"], true)]])]),
      win("Empty", [usage("empty(text)", "say what is missing in a sentence, inside the window that would hold it."), empty("No evaluations recorded for this loop.")]),
    ),
    cols(
      win("Counters", [usage("counters(items)", "headline numbers, each linking to its page. At most one block per page."), counters([{ n: 23, label: "Loops", detail: "10 specified" }, { n: 4, label: "Gaps", detail: "no loop yet" }])], { flag: SAMPLE }),
      win("Gauge and split", [
        usage("gauge(label, n, total, tone)", "one real fraction. Never for an estimate."),
        gauge("Gates pass", 6, 6, "good"),
        gauge("k-rust built", 1, 10, "critical"),
        usage("split(parts, caption?)", "one total broken down by status: bar plus labelled counts."),
        split([{ label: "pass", n: 59, tone: "good" }, { label: "partial", n: 2, tone: "warning" }, { label: "unclassified", n: 12, tone: "neutral" }], "Current reference"),
      ], { flag: SAMPLE }),
    ),
    win("Slot matrix", [
      usage("slotMatrix(grids, legend, start)", "a universe of 16 × 16 slots: legend, then grids. The slot under the cursor is read out on the info screen (its detail, under its key and status). The D-pad moves the cursor across grids. critical slots carry ✕; marked slots are structural (a prefix)."),
      slotMatrix(grids, [{ tone: "good", label: "pass", n: 250 }, { tone: "critical", glyph: "✕", label: "fail", n: 2 }, { tone: "neutral", label: "no vectors", n: 9 }, { tone: "neutral", marked: true, glyph: "CB", label: "prefix", n: 1 }], [0, 0x10]),
      usage("miniGrid(grid, caption), slotLegend(items, stacked)", "a small read-only map for overviews."),
      cols(miniGrid(grids[0]!, "Grid A"), slotLegend([{ tone: "good", label: "pass", n: 250, meaning: "every vector passes" }, { tone: "critical", glyph: "✕", label: "fail", n: 2, meaning: "a vector fails" }], true)),
    ], { flag: SAMPLE }),
    cols(
  win("Record chart", [usage("recordChart({ points, better, format, summary })", "one value per run over time; the step line is the best so far. Fits half a screen (inside cols)."), recordChart({ points: runs, better: "lower", format: (v) => `${v.toLocaleString("en")}k`, summary: "Sample: best value falls from 1,284k to 1,121k over 18 runs" })], { flag: SAMPLE }),
      win("Chart rules", list([
        "Only the chart kinds in ui/charts.ts. A new kind is a new builder there plus a section here.",
        "Every chart has a summary sentence for screen readers.",
        "A series is told apart by colour and by marker shape; a status is never a series colour.",
        "Anything not measured from tracking files carries the window flag SAMPLE DATA or MOCK DATA.",
        "Pixel style: crisp edges, step lines, square markers, no smoothing, no gradients.",
      ])),
    ),
    win("Frontier chart", [
        usage("frontierChart({ series, xLabel, yLabel, summary })", "up to four series where lower-left is better; each series has a colour and a marker shape. Give it a full-width window: its labels are sized for one."),
        frontierChart({
          series: [
            { name: "ALU", points: [[18, 9.5], [22, 7.2], [26, 6.0], [31, 5.1], [38, 4.8], [24, 8.4], [29, 6.9]] },
            { name: "load/store", points: [[20, 13.0], [24, 11.2], [30, 9.6], [37, 8.9], [28, 12.4]] },
            { name: "control", points: [[16, 16.5], [19, 14.0], [25, 12.8], [32, 12.1], [22, 15.6]] },
            { name: "stack", points: [[21, 19.0], [27, 17.2], [33, 15.9], [40, 15.0], [30, 18.4]] },
          ],
          xLabel: "trace width (columns)",
          yLabel: "rows per operation",
          summary: "Sample: four lanes of width against rows per operation",
        }),
      ], { flag: SAMPLE }),
  ];
}

export function guideView(m: Model) {
  return page({ title: "Style guide", sub: "every token and component, drawn by the code views use; rules in site/DESIGN.md" }, tokens(), sprites(), components(m));
}
