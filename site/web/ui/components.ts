// Screen components. Views compose these; they do not write component markup or class names themselves.
// Each builder matches one block in styles/components.css and one section of the #/guide route.
// See DESIGN.md, "Components", for when to use which.

import { countBy, h, type Child } from "./dom";
import { status } from "./status";
import type { Tone } from "./sprites";
import { toneSprite } from "./sprites";

// ---------- routes and links ----------

/** A link to a route of this site: route("loops", id) is "#/loops/<id>". */
export const route = (...parts: string[]) => "#/" + parts.map(encodeURIComponent).join("/");
export const link = (href: string, text: Child, cls = "id") => h("a", { href, class: cls }, text);

// ---------- page structure ----------

export interface PageHead {
  title: string;
  sub?: Child;
  /** A line above the title linking back to the parent page, for detail routes. */
  crumb?: { href: string; label: string };
}

/** A whole route: title row, then blocks top to bottom. Every view returns exactly one page(). */
export function page(head: PageHead, ...blocks: Child[]): HTMLElement {
  return h(
    "div",
    { class: "page" },
    h("header", { class: "page-title" }, head.crumb ? h("a", { class: "crumb", href: head.crumb.href }, "◀ ", head.crumb.label) : null, h("h1", {}, head.title), head.sub ? h("span", { class: "sub" }, head.sub) : null),
    blocks,
  );
}

/** Blocks side by side that wrap to a stack when the screen is narrow. Use for two or three related windows. */
export const cols = (...blocks: Child[]) => h("div", { class: "cols" }, blocks);

export interface WinOptions {
  /** Short text at the right of the title bar: a count, a unit, a date. */
  aside?: Child;
  /** Marks synthetic or provisional content; shown in the warning colour at the right of the title bar. */
  flag?: string;
  id?: string;
}

/** A window: the framed box every block of content sits in. Title is a short noun phrase, shown in capitals. */
export function win(title: Child, body: Child, opts: WinOptions = {}): HTMLElement {
  return h(
    "section",
    { class: "win", id: opts.id },
    h("div", { class: "win-title" }, h("span", {}, title), opts.flag ? h("span", { class: "aside flag" }, opts.flag) : opts.aside ? h("span", { class: "aside" }, opts.aside) : null),
    h("div", { class: "win-body" }, body),
  );
}

/** A dialogue: a message to the reader about the data (not a block of data). Give a tone when it reports a status. */
export function dialogue(body: Child, tone?: Tone): HTMLElement {
  return h("div", { class: "win dialogue", role: tone && tone !== "good" ? "note" : undefined }, tone ? toneSprite(tone) : null, h("div", {}, body));
}

/** Text for an empty block: say what is missing, in a sentence. */
export const empty = (text: string) => h("p", { class: "empty" }, text);
export const muted = (text: Child) => h("span", { class: "muted" }, text);
export const small = (text: Child) => h("span", { class: "small muted" }, text);

// ---------- labels ----------

/** A plain label in a frame: a shape name, a role such as "baseline". Not for status; use status() for anything with a tone. */
export const tag = (text: string, quiet = false) => h("span", { class: quiet ? "tag quiet" : "tag" }, text);

/** Several statuses in a row that wraps. */
export const chips = (items: Child[]) => h("div", { class: "chips" }, items);

/** Counts of items by status, each as a status with its count. */
export function statusCounts<T>(items: T[], key: (item: T) => string, toneOf: (id: string) => Tone): HTMLElement {
  return chips(countBy(items, key).map(([id, n]) => status(toneOf(id), `${id} ${n}`)));
}

// ---------- tables ----------

export interface Column {
  label: string;
  /** Right-aligned tabular numbers. */
  num?: boolean;
  /** Keep the cell on one line (ids). */
  nowrap?: boolean;
}

export interface Row {
  cells: Child[];
  /** Row id, so #/route/<id> can select it. */
  id?: string;
  /** Where A (or a click) on this row goes. */
  href?: string;
  /** Rows are selectable with the D-pad unless this is false. */
  select?: boolean;
}

/** A table. Rows are selectable by default: the D-pad moves the cursor and A opens `href`. */
export function table(columns: Column[], rows: Row[]): HTMLElement {
  return h(
    "div",
    { class: "table-wrap" },
    h(
      "table",
      { class: "table" },
      h("thead", {}, h("tr", {}, columns.map((c) => h("th", { class: c.num ? "num" : undefined }, c.label)))),
      h(
        "tbody",
        {},
        rows.map((r) =>
          h(
            "tr",
            { "data-sel": r.select === false ? undefined : true, "data-id": r.id, "data-href": r.href },
            r.cells.map((cell, i) => h("td", { class: [columns[i]?.num ? "num" : "", columns[i]?.nowrap ? "nowrap" : ""].join(" ").trim() || undefined }, cell)),
          ),
        ),
      ),
    ),
  );
}

// ---------- fields ----------

/** Label / value pairs. A value of undefined renders as "not specified" so missing fields stay visible. */
export function fields(pairs: [string, Child | undefined][]): HTMLElement {
  return h("dl", { class: "fields" }, pairs.map(([label, value]) => [h("dt", {}, label), h("dd", {}, value === undefined || value === "" || (Array.isArray(value) && value.length === 0) ? muted("not specified") : value)]));
}

export const list = (items: Child[], ordered = false) => h(ordered ? "ol" : "ul", { class: "list" }, items.map((i) => h("li", {}, i)));

// ---------- numbers ----------

export interface Counter {
  n: Child;
  label: string;
  detail?: Child;
  href?: string;
}

/** Headline numbers. Use on overview-style pages, at most one counters block per page. */
export const counters = (items: Counter[]) =>
  h("div", { class: "counters" }, items.map((c) => h(c.href ? "a" : "div", { class: "counter", href: c.href }, h("span", { class: "n" }, c.n), h("span", { class: "l" }, c.label), c.detail !== undefined ? h("span", { class: "d" }, c.detail) : null)));

/** One real fraction: n of total, as a segmented bar with the numbers beside it. Never for estimates.
 *  `tone` is the tone of the status being counted (counting passes: "good"), never a judgement of how big the fraction is. */
export function gauge(label: string, n: number, total: number, tone: Tone): HTMLElement {
  const width = total ? (n / total) * 100 : 0;
  return h("div", { class: "gauge" }, h("span", {}, label), h("div", { class: "bar", role: "img", "aria-label": `${label}: ${n} of ${total}` }, h("i", { class: `fill-${tone}`, style: `width:${width.toFixed(1)}%` })), h("span", {}, `${n}/${total}`));
}

export interface Part {
  label: string;
  n: number;
  tone: Tone;
}

/** One total broken down by status: a proportional bar plus the same parts as labelled statuses. */
export function split(parts: Part[], caption?: Child): HTMLElement {
  const text = parts.map((p) => `${p.n} ${p.label}`).join(", ");
  return h(
    "div",
    { class: "split" },
    caption ? h("span", { class: "small muted" }, caption) : null,
    h("div", { class: "split-bar", role: "img", "aria-label": text }, parts.map((p) => h("i", { class: `fill-${p.tone}`, style: `flex:${p.n}` }))),
    chips(parts.map((p) => status(p.tone, `${p.label} ${p.n}`))),
  );
}

// ---------- prose ----------

/** Rendered Markdown from a trusted local repository file. The only place innerHTML is allowed. */
export function prose(html: string, rewrite?: (a: HTMLAnchorElement) => void): HTMLElement {
  const el = h("div", { class: "prose" });
  el.innerHTML = html;
  if (rewrite) el.querySelectorAll("a[href]").forEach((a) => rewrite(a as HTMLAnchorElement));
  return el;
}
