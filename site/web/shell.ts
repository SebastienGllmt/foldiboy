// The shell's behaviour: the hardware buttons, their keys, the row cursor, the deck's page list and the LIVE light.
// Views never handle keys or buttons; they mark selectable rows (table rows do this) and the shell does the rest.
// Button contract (also printed on the control half and in the button tooltips):
//   D-pad ← →   previous / next page in the menu (on a slot matrix: move the slot cursor)
//   D-pad ↑ ↓   previous / next selectable row, read out on the info screen (on a slot matrix: move the slot cursor;
//               with no rows: scroll; with the deck's page list open: move through it)
//   A  (X)      open the selected row's link (page list: go to the page)
//   B  (Z)      back (flip fold: first closes the page list, then clears the selected row)
//   START (Enter)   overview (flip fold: open or close the page list)
//   SELECT (Shift)  jump to the next window on the page
// See DESIGN.md, "Controls".

import { rowReadout, showInfo } from "./ui/info";
import { moveCursor } from "./ui/matrix";

export type Key = "up" | "down" | "left" | "right" | "a" | "b" | "start" | "select";

export interface ShellActions {
  /** Go to the page `delta` steps away in the menu order. */
  page(delta: number): void;
  start(): void;
}

/** The flip fold (main screen over a control deck) is used below this width; keep in step with shell.css. */
const flip = matchMedia("(max-width: 719px)");
export const isFlip = () => flip.matches;

const screen = () => document.getElementById("screen")!;
const rows = () => [...screen().querySelectorAll<HTMLTableRowElement>("tr[data-sel]")];
let cursor = -1;

/** Called by the router after each render; `select` is the row id named in the URL, if any. */
export function resetCursor(select?: string) {
  cursor = -1;
  setDeckList(false);
  if (select) {
    const i = rows().findIndex((r) => r.dataset.id === select);
    if (i >= 0) setRow(i, "center");
  }
}

function setRow(i: number, block: ScrollLogicalPosition = "nearest") {
  const all = rows();
  if (!all.length) return;
  cursor = Math.max(0, Math.min(all.length - 1, i));
  all.forEach((r, k) => r.classList.toggle("cur", k === cursor));
  const row = all[cursor]!;
  setDeckList(false);
  showInfo(rowReadout(row));
  row.scrollIntoView({ block });
}

function clearRow() {
  cursor = -1;
  rows().forEach((r) => r.classList.remove("cur"));
  showInfo(null);
}

// ---------- the deck's page list (flip fold): the second screen shows the menu instead of the info ----------
const deckOpen = () => document.body.dataset.deck === "list";
const menuLinks = () => [...document.querySelectorAll<HTMLAnchorElement>("#menu a")];
export function setDeckList(open: boolean) {
  document.body.dataset.deck = open ? "list" : "";
  document.querySelector("#deckline .name")?.setAttribute("aria-expanded", String(open));
  const links = menuLinks();
  links.forEach((a) => a.classList.remove("hi"));
  if (open) links.find((a) => a.hasAttribute("aria-current"))?.classList.add("hi");
}
function moveDeckList(delta: number) {
  const links = menuLinks();
  const i = Math.max(0, links.findIndex((a) => a.classList.contains("hi")));
  const next = (i + delta + links.length) % links.length;
  links.forEach((a, k) => a.classList.toggle("hi", k === next));
  links[next]?.scrollIntoView({ block: "nearest" });
}

function nextWindow() {
  const s = screen();
  const top = s.getBoundingClientRect().top + (s.querySelector<HTMLElement>(".titlebar")?.offsetHeight ?? 0) + 8;
  const blocks = [...s.querySelectorAll<HTMLElement>(".page > .win, .page > .cols, .page > section")];
  const next = blocks.find((b) => b.getBoundingClientRect().top > top + 4);
  s.scrollTo({ top: next ? s.scrollTop + next.getBoundingClientRect().top - top : 0, behavior: "smooth" });
}

export function press(key: Key, actions: ShellActions) {
  const button = document.querySelector(`.half.ctl [data-key="${key}"]`);
  if (button) {
    button.classList.add("pressed");
    setTimeout(() => button.classList.remove("pressed"), 120);
  }
  if (deckOpen() && (key === "up" || key === "down" || key === "a")) {
    if (key === "a") {
      const hi = menuLinks().find((a) => a.classList.contains("hi"));
      if (hi) location.hash = hi.getAttribute("href")!;
      setDeckList(false);
    } else moveDeckList(key === "up" ? -1 : 1);
    return;
  }
  switch (key) {
    case "left":
    case "right":
      if (!moveCursor(key === "left" ? -1 : 1, 0)) actions.page(key === "left" ? -1 : 1);
      return;
    case "up":
    case "down": {
      if (moveCursor(0, key === "up" ? -1 : 1)) return;
      if (rows().length) setRow(cursor + (key === "up" ? -1 : 1));
      else screen().scrollBy({ top: key === "up" ? -120 : 120, behavior: "smooth" });
      return;
    }
    case "a": {
      const row = rows()[cursor];
      if (row?.dataset.href) location.hash = row.dataset.href;
      else row?.animate([{ filter: "invert(1)" }, { filter: "none" }], { duration: 160 });
      return;
    }
    case "b":
      if (deckOpen()) setDeckList(false);
      else if (isFlip() && cursor >= 0) clearRow();
      else history.back();
      return;
    case "start":
      if (isFlip()) setDeckList(!deckOpen());
      else actions.start();
      return;
    case "select":
      nextWindow();
      return;
  }
}

const KEYS: Record<string, Key> = { ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down", x: "a", X: "a", z: "b", Z: "b", Enter: "start" };

export function installControls(actions: ShellActions) {
  for (const b of document.querySelectorAll<HTMLElement>(".half.ctl [data-key]")) b.addEventListener("click", () => press(b.dataset.key as Key, actions));
  // Clicking a row selects it and reads it out; following its link is A's job (or the link inside it).
  screen().addEventListener("click", (e) => {
    const tr = (e.target as Element).closest?.("tr[data-sel]");
    if (tr && !(e.target as Element).closest("a")) setRow(rows().indexOf(tr as HTMLTableRowElement));
  });
  flip.addEventListener("change", () => setDeckList(false));
  // SELECT is Shift pressed and released on its own, so Shift+click and capitals never trigger it.
  let shiftAlone = false;
  document.addEventListener("keydown", (e) => {
    if (e.key === "Shift") { shiftAlone = !e.repeat; return; }
    shiftAlone = false;
    if (e.altKey || e.ctrlKey || e.metaKey) return;
    const target = e.target as Element;
    if (e.key === "Enter" && target.closest?.("a, button")) return; // Enter on a focused link or button keeps its meaning
    const key = KEYS[e.key];
    if (!key) return;
    e.preventDefault();
    press(key, actions);
  });
  document.addEventListener("keyup", (e) => {
    if (e.key === "Shift" && shiftAlone) press("select", actions);
    shiftAlone = false;
  });
}

/** The LIVE light: on while the server's event stream is connected, flashes when the model reloads, off when disconnected. */
export function led(state: "live" | "off" | "flash") {
  const el = document.querySelector<HTMLElement>(".led")!;
  el.dataset.state = state;
  el.title = state === "off" ? "Not connected to the tracking server" : "Live: the screen updates when a tracking file changes";
  if (state === "flash") setTimeout(() => led("live"), 400);
}
