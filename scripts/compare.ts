// Compares the Rust emulator's CPU with the SM83 definition on every corpus case:
//
//   bun scripts/compare.ts
//
// 1. Every case is written into one case document (corpus/build/cases.json): a cartridge of
//    corpus/cases with no interrupt request, and each case document of corpus/scheduled as written.
// 2. `krs run --case cases.json#ID --depth 0 --output end` writes each case's start as the
//    definition builds it (corpus/build/starts/).
// 3. krs/sm83-rust-cpu runs the emulator's CPU from each start and writes its end as a krs end
//    document (corpus/build/ends/).
// 4. One `krs compare --pairs` runs each case on the definition and compares the two ends on
//    every cell, the bus log under each view of krs/sm83-views:
//
//   WritesView    the writes, in order. Required equal, except where the emulator delivered a
//                 request late (below).
//   AccessesView  the reads and writes, in order. Reported only: the emulator performs an
//                 instruction's accesses at once, and the definition's interrupt dispatch starts
//                 with a discarded opcode read the emulator does not make.
//
// The emulator executes an instruction or a dispatch at once, so it delivers a request only between
// two of them: one whose cycle falls inside them comes late (corpus/build/ends/late.json), and that
// case is reported only.
//
// Exits 1 when a case is not equal under WritesView.

import { readdir, rm } from "node:fs/promises";
import { resolve } from "node:path";
import { $ } from "bun";
import { krs, printSummary, root } from "./krs";

const VIEWS = ["WritesView", "AccessesView"];
const REQUIRED = "WritesView";
const JOBS = 4;

const build = resolve(root, "corpus/build");
await rm(build, { recursive: true, force: true });

// 1. The cases. A case document's `$file` is relative to its directory.
type Case = { name: string; inputs: Record<string, unknown> };
const cases: Case[] = [];
for (const id of (await readdir(resolve(root, "corpus/cases"))).sort()) {
  const cartridge = { $file: resolve(root, "corpus/cases", id, "case.gb") };
  cases.push({ name: id, inputs: { cartridge, requests: [] } });
}
for (const id of (await readdir(resolve(root, "corpus/scheduled"))).sort()) {
  const dir = resolve(root, "corpus/scheduled", id);
  const document = Bun.TOML.parse(await Bun.file(`${dir}/case.toml`).text()) as { inputs: Record<string, any> };
  const inputs = { ...document.inputs, cartridge: { $file: resolve(dir, document.inputs.cartridge.$file) } };
  cases.push({ name: id, inputs });
}
const ids = cases.map((c) => c.name);
// A case that enters HALT mode with nothing to wake it stops at its second halted boundary, as in
// the corpus (FLOW.toml [stop]); no other case reaches one.
const run = { stop_before: ["crate::interrupt::boundary_halted"], stop_count: 2 };
await Bun.write(`${build}/cases.json`, JSON.stringify({ run, case: cases }));

// 2. The starts, JOBS at a time.
const queue = [...ids];
async function starts() {
  for (let id = queue.shift(); id; id = queue.shift()) {
    const status = await krs("run", ["--case", `${build}/cases.json#${id}`, "--depth", "0", "--output", "end"], `corpus/build/starts/${id}.json`);
    if (status !== 0) throw new Error(`${id}: krs run exited ${status}`);
  }
}
await Promise.all(Array.from({ length: JOBS }, starts));

// 3. The emulator's ends.
await $`cargo build --release --offline --quiet`.cwd(resolve(root, "krs/sm83-rust-cpu"));
await $`${root}/krs/sm83-rust-cpu/target/release/sm83-rust-cpu --out ${build}/ends ${ids.map((id) => `${build}/starts/${id}.json`)}`;
const late: Record<string, number> = await Bun.file(`${build}/ends/late.json`).json();

// 4. Every case under every view, in one batch.
const pairs = ids.flatMap((id) =>
  VIEWS.map((view) => ({ left: `cases.json#${id}`, right: `ends/${id}.json`, views: { bus: view }, observer: "foldiboy-sm83-views:views" })),
);
await Bun.write(`${build}/pairs.json`, JSON.stringify({ pair: pairs }, null, 2));
await krs("compare", ["--pairs", `${build}/pairs.json`, "--jobs", String(JOBS), "--output", "json"], "corpus/build/compare.jsonl");

const records = (await Bun.file(`${build}/compare.jsonl`).text()).trim().split("\n").map((line) => JSON.parse(line));
const failed: string[] = [];
pairs.forEach((pair, index) => {
  const id = ids[Math.floor(index / VIEWS.length)]!;
  const result = records.find((record) => record.pair?.number === index + 1) ?? { verdict: "missing" };
  const at = result.difference?.path ? ` at ${result.difference.path}` : "";
  const note = late[id] ? ` (the emulator delivered ${late[id]} request(s) late)` : "";
  console.log(`${id} ${pair.views.bus}: ${result.verdict}${at}${note}`);
  if (pair.views.bus === REQUIRED && result.verdict !== "equal" && !late[id]) failed.push(id);
});
await printSummary("corpus/build/compare.jsonl");
if (failed.length) {
  console.log(`not equal under ${REQUIRED}: ${failed.join(", ")}`);
  process.exit(1);
}
