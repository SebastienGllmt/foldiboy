# Getting started with Foldiboy

Foldiboy is a Game Boy emulator packaged as a Wasm Component, with a native desktop host and an SM83 semantics model in KRS.
The emulator and proving architecture are developed together; design documents describe requirements and open questions as well as implemented behavior.

## Read the documentation site

From the repository root:

```sh
cd site
bun install --frozen-lockfile
bun run dev
```

Open <http://127.0.0.1:4173/#/docs>.
The Docs menu lists the repository guides, managed design documents and AI-assisted documentation.
Each page renders its source Markdown directly; editing a document refreshes the open site.
The site runs locally and has no configured public deployment.
See the [site guide](../site/README.md) for configuration.

## Run the emulator

Use the Rust toolchain and Wasm tooling described in the [project README](../README.md#building--running).
On this machine, follow the machine setup policy before installing missing tools.
From the repository root:

```sh
cd host/desktop
./build.sh
```

Place game ROM files inside `roms/` before selecting one in the ROM browser.
The WASI filesystem sandbox cannot follow symlinks outside that directory.
Arrow keys control the D-pad, X or J presses A, Z or K presses B, Enter presses Start, and Right Shift presses Select.
Escape switches focus between the emulator and its debug panels.
See the [desktop guide](../host/desktop/README.md) for host diagnostics and display settings.

## Run tests

From the repository root:

```sh
bun run test
```

Clone `retrio/gb-test-roms` and `LIJI32/SameSuite` into the suite directory before running tests; `bun run test` builds SameSuite and runs the emulator tests.
The default ROM-suite location is `../roms/tests/` relative to the repository's workspace path.
Set `FOLDIBOY_TEST_ROOT` when your test suites live elsewhere.
See the [testing guide](../machine-docs/testing.md) for suite coverage, optional tests and accuracy limitations.

## Understand the architecture

Start with the [project overview](../README.md#architecture) for the guest, host and WIT interfaces.
The [ideal emulator design](ideal.md) describes the intended proving-oriented architecture.
The [provability design](provability.md) specifies the zk-cycle model and opcode expansion, while [constraints and lookups](constraints.md) describes arithmetic tables, flag constraints and unresolved choices.
The [AI-assisted architecture](../machine-docs/ideal.md) and [provability analysis](../machine-docs/provability.md) expand those designs.
These documents must not be treated as evidence that every described proving feature is implemented.

## Work on SM83 semantics

The [KRS guide](../krs/sm83/README.md) defines the state model, source layout, conventions, commands and conformance suites.
Research loops, tickets, coverage and toolchain capabilities are available through the site's other menu pages.
Their tracking data currently lives in `draft-personal/autoresearch/`; `FOLDIBOY_TRACKING_DIR` selects an alternative directory.
