// Client entry: fetches the tracking model, renders the route named in the URL hash into the screen, and keeps it live.
// Routes: #/<route> is a route's list page; #/<route>/<record> is its detail page, or the list with that record's row selected.

import type { Model } from "../src/model";
import { h } from "./ui/dom";
import { installControls, led, resetCursor } from "./shell";
import { ROUTES } from "./views/routes";

let model: Model | undefined;
const menuRoutes = ROUTES.filter((r) => r.menu);

function parseHash(): { id: string; record?: string } {
  const [id = "overview", record] = location.hash.replace(/^#\/?/, "").split("/").map(decodeURIComponent);
  return { id: ROUTES.some((r) => r.id === id) ? id : "overview", record: record || undefined };
}

function menu(current: string, m: Model) {
  return h(
    "nav",
    { class: "menu", "aria-label": "Pages" },
    menuRoutes.map((r) => {
      const n = r.count?.(m);
      return h("a", { href: `#/${r.id}`, "aria-current": r.id === current ? "page" : undefined }, r.label, n !== undefined ? h("span", { class: "count" }, String(n)) : null);
    }),
    h("span", { class: "keys", title: "Arrow keys page and select, X opens, Z goes back, Enter returns to the overview, Shift jumps to the next window" }, "←→ page · ↑↓ row · X open · Z back"),
    h("span", { class: "end" }, h("a", { href: "#/guide", "aria-current": current === "guide" ? "page" : undefined }, "style guide"), h("span", { title: "When the tracking model was loaded" }, "loaded " + new Date(m.loadedAt).toLocaleTimeString("en", { hour: "2-digit", minute: "2-digit" }))),
  );
}

function render(keepScroll = false) {
  if (!model) return;
  const { id, record } = parseHash();
  const r = ROUTES.find((x) => x.id === id)!;
  const screen = document.getElementById("screen")!;
  const scroll = screen.scrollTop;
  const body = record && r.detail ? r.detail(model, record) : r.list(model);
  screen.replaceChildren(menu(id, model), body);
  screen.scrollTop = keepScroll ? scroll : 0;
  resetCursor(r.detail ? undefined : record);
  document.title = `${r.label} · Foldiboy tracking`;
}

installControls({
  page(delta) {
    const i = menuRoutes.findIndex((r) => r.id === parseHash().id);
    location.hash = `#/${menuRoutes[(Math.max(0, i) + delta + menuRoutes.length) % menuRoutes.length]!.id}`;
  },
  start() {
    location.hash = "#/overview";
  },
});

async function refresh(keepScroll: boolean) {
  model = (await (await fetch("/api/model")).json()) as Model;
  render(keepScroll);
}

window.addEventListener("hashchange", () => render());

// Tracking-file changes re-render in place; a reconnect means the server restarted with new site code, so reload the page.
const events = new EventSource("/api/events");
let connected = false;
events.addEventListener("open", () => {
  if (connected) location.reload();
  connected = true;
  led("live");
});
events.addEventListener("error", () => led("off"));
events.addEventListener("model", () => {
  led("flash");
  void refresh(true);
});
led("off");
void refresh(false);
