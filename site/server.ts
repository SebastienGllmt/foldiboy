// Local tracking site: serves the web client and pushes a reload event when a tracking file changes.

import { watch, type FSWatcher } from "node:fs";
import index from "./web/index.html";
import { loadModel, trackingDir, type Model } from "./src/model";

const dir = trackingDir();
const encoder = new TextEncoder();
const clients = new Set<ReadableStreamDefaultController<Uint8Array>>();
let model: Model = await loadModel(dir);

function broadcast(event: string) {
  for (const client of clients) {
    try {
      client.enqueue(encoder.encode(`event: ${event}\ndata: ${model.loadedAt}\n\n`));
    } catch {
      clients.delete(client);
    }
  }
}

// `bun --hot` re-evaluates this module; replace the previous watcher rather than adding another.
const state = globalThis as { trackingWatcher?: FSWatcher; reloadTimer?: Timer };
state.trackingWatcher?.close();
state.trackingWatcher = watch(dir, { recursive: true }, () => {
  clearTimeout(state.reloadTimer);
  state.reloadTimer = setTimeout(async () => {
    model = await loadModel(dir);
    broadcast("model");
  }, 100);
});

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
