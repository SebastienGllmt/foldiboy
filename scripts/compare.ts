// Compares the Rust emulator's CPU with the SM83 definition on every corpus case:
//
//   bun scripts/compare.ts
//
// 1. `krs run CARTRIDGE --depth 0 --output end` writes each case's start as the definition builds it
//    (corpus/build/starts/).
// 2. krs/sm83-rust-cpu runs the emulator's CPU from each start and writes its end as a krs end
//    document (corpus/build/ends/).
// 3. One `krs compare --pairs` runs each case on the definition and compares the two ends on
//    every cell, the bus log under each view of krs/sm83-views:
//
//   WritesView    the writes, in order. Required equal.
//   AccessesView  the reads and writes, in order. Reported only: the emulator performs an
//                 instruction's accesses at once, and the definition's interrupt dispatch starts
//                 with a discarded opcode read the emulator does not make.
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
const ids = (await readdir(resolve(root, "corpus/cases"))).sort();
const cartridge = (id: string) => resolve(root, "corpus/cases", id, "case.gb");
await rm(build, { recursive: true, force: true });

// 1. The starts, JOBS at a time.
const queue = [...ids];
async function starts() {
  for (let id = queue.shift(); id; id = queue.shift()) {
    const status = await krs("run", [cartridge(id), "--depth", "0", "--output", "end"], `corpus/build/starts/${id}.json`);
    if (status !== 0) throw new Error(`${id}: krs run exited ${status}`);
  }
}
await Promise.all(Array.from({ length: JOBS }, starts));

// 2. The emulator's ends.
await $`cargo build --release --offline --quiet`.cwd(resolve(root, "krs/sm83-rust-cpu"));
await $`${root}/krs/sm83-rust-cpu/target/release/sm83-rust-cpu --out ${build}/ends ${ids.map((id) => `${build}/starts/${id}.json`)}`;

// 3. Every case under every view, in one batch. A case that enters HALT mode stops at its second
//    halted boundary, as in the corpus (FLOW.toml [stop]); no other case reaches one.
const run = { stop_before: ["crate::interrupt::boundary_halted"], stop_count: 2 };
const cases = ids.map((id) => ({ name: id, inputs: { cartridge: { $file: cartridge(id) } } }));
await Bun.write(`${build}/cases.json`, JSON.stringify({ run, case: cases }));
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
  console.log(`${id} ${pair.views.bus}: ${result.verdict}${at}`);
  if (pair.views.bus === REQUIRED && result.verdict !== "equal") failed.push(id);
});
await printSummary("corpus/build/compare.jsonl");
if (failed.length) {
  console.log(`not equal under ${REQUIRED}: ${failed.join(", ")}`);
  process.exit(1);
}
