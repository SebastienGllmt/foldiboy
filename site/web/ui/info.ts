// The info screen: the panel under the menu on the device's second screen. It reads out what the cursor is on.
// The router sets the page's own description (setPageInfo); the shell shows the selected row (rowReadout); the slot matrix shows
// the slot under its cursor. showInfo(null) goes back to the page description. See DESIGN.md, "The info screen".

import { h, type Child } from "./dom";

let pageContent: Node[] = [];
const panel = () => document.getElementById("info");

/** Replaces what the info screen shows; null shows the page's description again. */
export function showInfo(content: Node[] | null) {
  const p = panel();
  if (!p) return;
  p.replaceChildren(...(content ?? pageContent));
  p.scrollTop = 0;
}

/** Sets the page's description, shown whenever nothing is selected, and shows it. */
export function setPageInfo(content: Node[]) {
  pageContent = content;
  showInfo(null);
}

/** The bar at the top of a readout: what is being read out, and optionally its status. */
export const infoTitle = (...children: Child[]) => h("div", { class: "info-title" }, children);

/** A table row read back as its column heads and cell contents. The title is the row's id, else its first two cells. */
export function rowReadout(tr: HTMLTableRowElement): Node[] {
  const heads = [...tr.closest("table")!.querySelectorAll("thead th")].map((th) => th.textContent!.trim());
  const cells = [...tr.children] as HTMLElement[];
  const title = tr.dataset.id ?? cells.slice(0, 2).map((td) => td.textContent!.trim()).filter(Boolean).join(" · ");
  const pairs = cells.map((td, i) => [heads[i] ?? "", td] as const).filter(([, td]) => td.textContent!.trim() && td.textContent!.trim() !== tr.dataset.id);
  return [
    infoTitle(title),
    h("dl", { class: "fields" }, pairs.map(([label, td]) => [h("dt", {}, label), h("dd", {}, [...td.cloneNode(true).childNodes])])),
    tr.dataset.href ? h("a", { class: "open", href: tr.dataset.href }, h("b", {}, "A"), " open ", decodeURIComponent(tr.dataset.href.split("/").pop()!)) : null,
  ].filter((x): x is HTMLElement => x !== null);
}
