// Client entry: fetches the tracking model, renders the route named in the URL hash into the main screen (with its title bar),
// fills the second screen (menu, the deck's page line, the page's description on the info screen), and keeps it live.
// Routes: #/<route> is a route's list page; #/<route>/<record> is its detail page, or the list with that record's row selected.

import type { Model } from "../src/model";
import { h } from "./ui/dom";
import { setPageInfo } from "./ui/info";
import { installControls, led, resetCursor, setDeckList } from "./shell";
import { ROUTES, type Route } from "./views/routes";

let model: Model | undefined;
const menuRoutes = ROUTES.filter((r) => r.menu);

function parseHash(): { id: string; record?: string } {
  const [id = "overview", record] = location.hash.replace(/^#\/?/, "").split("/").map(decodeURIComponent);
  return { id: ROUTES.some((r) => r.id === id) ? id : "overview", record: record || undefined };
}

const loadedAt = (m: Model) => new Date(m.loadedAt).toLocaleTimeString("en", { hour: "2-digit", minute: "2-digit" });

/** The menu on the second screen: every menu route with its count. */
function menu(current: string, m: Model) {
  document.getElementById("menu")!.replaceChildren(
    ...menuRoutes.map((r) => {
      const n = r.count?.(m);
      return h("a", { href: `#/${r.id}`, "aria-current": r.id === current ? "page" : undefined }, h("span", {}, r.label), h("span", { class: "count" }, n !== undefined ? String(n) : ""));
    }),
  );
}

/** The deck's top line (flip fold only): ◀ page ▶, and the page name opens the page list. */
function deckline(r: Route) {
  const i = menuRoutes.indexOf(r);
  const button = (label: string, attrs: Record<string, string>, ...kids: (string | HTMLElement)[]) => h("button", { type: "button", "aria-label": label, ...attrs }, ...kids);
  const prev = button("Previous page", {}, "◀");
  const next = button("Next page", {}, "▶");
  const name = button("All pages (START)", { class: "name", "aria-expanded": "false" }, r.label, h("span", { class: "count" }, i >= 0 ? `${i + 1}/${menuRoutes.length}` : ""), h("span", { class: "caret", "aria-hidden": "true" }, "▾"));
  prev.addEventListener("click", () => actions.page(-1));
  next.addEventListener("click", () => actions.page(1));
  name.addEventListener("click", () => setDeckList(document.body.dataset.deck !== "list"));
  document.getElementById("deckline")!.replaceChildren(prev, name, next);
}

/** What the info screen shows when nothing is selected: the page, what it shows, and how to move around it. */
function pageInfo(r: Route, record: string | undefined, body: HTMLElement) {
  const hint = body.querySelector(".matrix[role=grid]") ? "D-pad moves the slot cursor · B back" : body.querySelector("tr[data-sel]") ? "↑↓ pick a row · ←→ page" : "↑↓ scroll · ←→ page";
  return [h("div", { class: "info-title about" }, record ?? r.label), h("p", {}, (record && r.detailAbout) || r.about), h("p", { class: "hint" }, hint)];
}

function render(keepScroll = false) {
  if (!model) return;
  const { id, record } = parseHash();
  const r = ROUTES.find((x) => x.id === id)!;
  const screen = document.getElementById("screen")!;
  const scroll = screen.scrollTop;
  const body = record && r.detail ? r.detail(model, record) : r.list(model);
  const titlebar = h("header", { class: "titlebar" }, body.querySelector(":scope > .page-title"), h("span", { class: "time", title: "When the tracking model was loaded" }, "loaded " + loadedAt(model)));
  screen.replaceChildren(titlebar, body);
  screen.scrollTop = keepScroll ? scroll : 0;
  menu(id, model);
  deckline(r);
  setPageInfo(pageInfo(r, record, body));
  resetCursor(r.detail ? undefined : record);
  document.title = `${r.label} · Foldiboy tracking`;
}

const actions = {
  page(delta: number) {
    const i = menuRoutes.findIndex((r) => r.id === parseHash().id);
    location.hash = `#/${menuRoutes[(Math.max(0, i) + delta + menuRoutes.length) % menuRoutes.length]!.id}`;
  },
  start() {
    location.hash = "#/overview";
  },
};
installControls(actions);

async function refresh(keepScroll: boolean) {
  model = (await (await fetch("/api/model")).json()) as Model;
  render(keepScroll);
}

window.addEventListener("hashchange", () => render());

// Live updates. "model": a tracking file changed, re-render in place. "reload": client code changed, reload the page and come
// back to the same scroll position. A reconnect means the server restarted, which may also mean new code, so reload too.
const SCROLL_KEY = "foldiboy-reload-scroll";
function reloadKeepingScroll() {
  sessionStorage.setItem(SCROLL_KEY, JSON.stringify({ hash: location.hash, top: document.getElementById("screen")!.scrollTop }));
  location.reload();
}
function restoreScroll() {
  const saved = sessionStorage.getItem(SCROLL_KEY);
  sessionStorage.removeItem(SCROLL_KEY);
  if (!saved) return;
  const { hash, top } = JSON.parse(saved) as { hash: string; top: number };
  if (hash === location.hash) document.getElementById("screen")!.scrollTop = top;
}

const events = new EventSource("/api/events");
let connected = false;
events.addEventListener("open", () => {
  if (connected) reloadKeepingScroll();
  connected = true;
  led("live");
});
events.addEventListener("error", () => led("off"));
events.addEventListener("reload", reloadKeepingScroll);
events.addEventListener("model", () => {
  led("flash");
  void refresh(true);
});
led("off");
void refresh(false).then(restoreScroll);
