# Foldiboy tracking site

A local site that renders the project's tracking data: loops, tickets, k-rust capabilities, vocabulary and the problems found in their cross-references.
It reads the tracking directory and never writes to it.

```bash
bun install
bun run dev      # http://127.0.0.1:4173, reloads when a tracking file or site source changes
bun run check    # prints problems; exits 1 on any error
bun run typecheck
```

`FOLDIBOY_TRACKING_DIR` selects the tracking directory; the default is `../draft-personal/autoresearch` until its files move into the repository (ticket T-DATA-HOME).
`PORT` selects the port.

| File | Content |
| --- | --- |
| `server.ts` | Bun server: the client with HMR, `/api/model`, and `/api/events`, which announces each reload of the tracking files |
| `src/model.ts` | Loads the tracking files, checks fields and references, derives reverse links |
| `src/check.ts` | Command-line problem report |
| `web/` | Client: the handheld shell, the design system in `web/ui/` and `web/styles/`, one view per route in `web/views/` |
| `DESIGN.md` | The design system: rules, tokens, components, controls, and how to extend it; read it before changing `web/` |

The site is styled as a handheld console screen; `DESIGN.md` explains the system, and `#/guide` (open it by URL; it is not linked from the site) shows every component.
