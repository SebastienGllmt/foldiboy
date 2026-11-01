// Slot matrix: one or more 16 × 16 grids of slots (for example opcode pages), a legend, and a readout of the slot under the cursor.
// The D-pad moves the cursor (shell.ts calls moveCursor); the cursor runs from one grid into the next at the right edge.
// See DESIGN.md, "Matrix".

import { h, type Child } from "./dom";
import { status } from "./status";
import type { Tone } from "./sprites";

export interface Slot {
  /** Short code shown on the axis-aligned readout title, e.g. "10" or "CB 7F". */
  key: string;
  /** Accessible name: what the slot is and its status. */
  label: string;
  tone: Tone;
  /** One character drawn in the slot; use it for the tones whose colour is not enough (critical: ✕). */
  glyph?: string;
  /** A structural slot that is not a member of the universe (a prefix, a reserved slot): drawn as an outlined box. */
  marked?: boolean;
  /** What the readout shows for this slot. */
  detail: Child;
}

export interface Grid {
  id: string;
  /** Text in the top-left corner of the grid, e.g. "CB". */
  corner?: string;
  /** 256 slots, row-major: slot y*16+x is at row y, column x. */
  slots: Slot[];
}

export interface LegendItem {
  tone: Tone;
  glyph?: string;
  marked?: boolean;
  label: string;
  n: number;
  meaning?: string;
}

const hex = (n: number) => n.toString(16).toUpperCase();
const slotClass = (tone: Tone, marked?: boolean) => `slot ${marked ? "marked" : `tone-${tone}`}`;

/** A legend for slot tones: a sample slot, a label, a count. stacked=true also shows each meaning. */
export function slotLegend(items: LegendItem[], stacked = false): HTMLElement {
  return h(
    "div",
    { class: stacked ? "legend stacked" : "legend" },
    items.map((i) =>
      h("div", { class: "item" }, h("span", { class: slotClass(i.tone, i.marked), "aria-hidden": "true" }, i.glyph ?? ""), stacked ? h("span", {}, i.label, i.meaning ? h("div", { class: "small muted" }, i.meaning) : null) : h("span", {}, i.label), h("span", {}, String(i.n))),
    ),
  );
}

/** A tiny non-interactive 16 × 16 map of a grid, for overviews. */
export function miniGrid(grid: Grid, caption: string): HTMLElement {
  const good = grid.slots.filter((x) => x.tone === "good").length;
  return h(
    "div",
    {},
    h("div", { class: "small muted" }, caption),
    h("div", { class: "matrix mini", role: "img", "aria-label": `${caption}: ${good} of ${grid.slots.length} good` }, grid.slots.map((x) => h("i", { class: slotClass(x.tone, x.marked) + " mini" }))),
  );
}

// The live matrix on the screen, if any; shell.ts moves its cursor.
let active: { grids: Grid[]; root: HTMLElement; readout: HTMLElement; cursor: [number, number] } | undefined;

function select(gi: number, code: number, scroll = true) {
  if (!active) return;
  const { grids, root, readout } = active;
  const grid = grids[gi]!;
  const slot = grid.slots[code]!;
  root.querySelectorAll(".slot.cur").forEach((x) => x.classList.remove("cur"));
  const el = root.querySelector<HTMLElement>(`[data-slot="${gi}:${code}"]`);
  el?.classList.add("cur");
  if (scroll) el?.scrollIntoView({ block: "nearest" });
  active.cursor = [gi, code];
  readout.replaceChildren(h("div", { class: "win-title" }, h("span", {}, "SLOT ", slot.key), h("span", { class: "aside" }, status(slot.tone, ""))), h("div", { class: "win-body" }, slot.detail));
}

/** Moves the cursor of the matrix on screen. Returns false when there is no matrix, so the shell can do its default. */
export function moveCursor(dx: number, dy: number): boolean {
  if (!active || !active.root.isConnected) return false;
  let [gi, code] = active.cursor;
  let x = (code % 16) + dx;
  let y = Math.floor(code / 16) + dy;
  if (x > 15 && gi < active.grids.length - 1) { gi++; x = 0; }
  else if (x < 0 && gi > 0) { gi--; x = 15; }
  x = Math.max(0, Math.min(15, x));
  y = Math.max(0, Math.min(15, y));
  select(gi, y * 16 + x);
  return true;
}

/** The interactive matrix: legend, grids side by side, readout. `start` is the slot the cursor starts on. */
export function slotMatrix(grids: Grid[], legend: LegendItem[], start: [number, number] = [0, 0]): HTMLElement {
  const readout = h("section", { class: "win readout", "aria-live": "polite" });
  const root = h(
    "div",
    { class: "matrix-block" },
    slotLegend(legend),
    h(
      "div",
      { class: "matrices" },
      grids.map((grid, gi) =>
        h(
          "div",
          { class: "matrix", role: "grid", "aria-label": `${grid.id} grid` },
          h("span", { class: "axis" }, grid.corner ?? ""),
          Array.from({ length: 16 }, (_, x) => h("span", { class: "axis" }, hex(x))),
          Array.from({ length: 16 }, (_, y) => [
            h("span", { class: "axis" }, hex(y) + "x"),
            Array.from({ length: 16 }, (_, x) => {
              const code = y * 16 + x;
              const slot = grid.slots[code]!;
              const b = h("button", { type: "button", class: slotClass(slot.tone, slot.marked), "data-slot": `${gi}:${code}`, "aria-label": slot.label, title: slot.label }, slot.glyph ?? "");
              b.addEventListener("mouseenter", () => select(gi, code, false));
              b.addEventListener("focus", () => select(gi, code, false));
              b.addEventListener("click", () => select(gi, code, false));
              return b;
            }),
          ]),
        ),
      ),
    ),
    readout,
  );
  active = { grids, root, readout, cursor: start };
  queueMicrotask(() => select(start[0], start[1], false));
  return root;
}
