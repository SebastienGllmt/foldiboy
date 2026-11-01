// Local tracking site: serves the web client and keeps open pages live.
// Events on /api/events: "model" when a tracking file changes (the page re-renders in place),
// "reload" when a client file under web/ changes (the page reloads, keeping its scroll position).

import { watch, type FSWatcher } from "node:fs";
import { resolve } from "node:path";
import index from "./web/index.html";
import { loadModel, trackingDir, type Model } from "./src/model";

type Client = ReadableStreamDefaultController<Uint8Array>;

// `bun --hot` re-evaluates this module on every server-side change. Connected pages and watchers live on globalThis so a
// re-evaluation keeps the pages that are already open and replaces the watchers instead of adding more.
const state = globalThis as {
  clients?: Set<Client>;
  trackingWatcher?: FSWatcher;
  webWatcher?: FSWatcher;
  reloadTimer?: Timer;
  webTimer?: Timer;
  evaluated?: boolean;
};

const dir = trackingDir();
const webDir = resolve(import.meta.dir, "web");
const encoder = new TextEncoder();
const clients = (state.clients ??= new Set<Client>());
let model: Model = await loadModel(dir);

function broadcast(event: "model" | "reload") {
  for (const client of clients) {
    try {
      client.enqueue(encoder.encode(`event: ${event}\ndata: ${model.loadedAt}\n\n`));
    } catch {
      clients.delete(client);
    }
  }
}

state.trackingWatcher?.close();
state.trackingWatcher = watch(dir, { recursive: true }, () => {
  clearTimeout(state.reloadTimer);
  state.reloadTimer = setTimeout(async () => {
    model = await loadModel(dir);
    broadcast("model");
  }, 100);
});

state.webWatcher?.close();
state.webWatcher = watch(webDir, { recursive: true }, () => {
  clearTimeout(state.webTimer);
  // Give Bun's development bundler a moment to pick up the change before pages ask for the new bundle.
  state.webTimer = setTimeout(() => broadcast("reload"), 150);
});

// A re-evaluation means server code changed: open pages reload to pick up anything it serves differently.
if (state.evaluated) setTimeout(() => broadcast("reload"), 150);
state.evaluated = true;

const server = Bun.serve({
  hostname: "127.0.0.1",
  port: Number(process.env.PORT ?? 4173),
  idleTimeout: 0,
  // Bun's browser HMR client throws in Firefox ("URL constructor:  is not a valid URL"); the client reloads itself instead.
  development: { hmr: false, console: true },
  routes: {
    "/": index,
    "/api/model": () => Response.json(model),
    "/api/events": (request) =>
      new Response(
        new ReadableStream<Uint8Array>({
          start(controller) {
            clients.add(controller);
            // Send something at once so the response headers flush and the client's LIVE light turns on without waiting for a heartbeat.
            controller.enqueue(encoder.encode(": connected\n\n"));
            const heartbeat = setInterval(() => controller.enqueue(encoder.encode(": heartbeat\n\n")), 15000);
            request.signal.addEventListener("abort", () => {
              clearInterval(heartbeat);
              clients.delete(controller);
            });
          },
        }),
        { headers: { "Content-Type": "text/event-stream", "Cache-Control": "no-cache" } },
      ),
  },
});

console.log(`tracking site at ${server.url} reading ${dir}`);
