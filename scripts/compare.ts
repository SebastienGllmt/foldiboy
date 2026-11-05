// Compares the Rust emulator's CPU with the SM83 definition on every corpus case:
//
//   bun scripts/compare.ts
//
// krs/sm83-rust-cpu runs the emulator's CPU from each case's start and writes its end as a krs
// end document (corpus/build/ends/). `krs compare` then runs the case on the definition and
// compares the two ends on every cell, the bus log under each view of krs/sm83-views:
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
import { krs, root } from "./krs";

const VIEWS = ["WritesView", "AccessesView"];
const REQUIRED = "WritesView";
const JOBS = 4;

const build = resolve(root, "corpus/build");
const ids = (await readdir(resolve(root, "corpus/cases"))).sort();
const caseFile = (id: string) => resolve(root, "corpus/cases", id, "case.toml");

// The emulator's ends. sm83-rust-cpu reads the definition's IR to evaluate a case's start.
await rm(build, { recursive: true, force: true });
await krs("emit-k", ["--ir", "--output", `${build}/ir`], "corpus/build/emit-k.log");
await $`cargo build --release --offline --quiet`.cwd(resolve(root, "krs/sm83-rust-cpu"));
await $`${root}/krs/sm83-rust-cpu/target/release/sm83-rust-cpu --definition-ir ${build}/ir/definition-ir.json --out ${build}/ends ${ids.map(caseFile)}`;

// One `krs compare` per case and view, JOBS at a time, each saved to corpus/build/compare/.
async function compare(id: string, view: string) {
  const args = [caseFile(id), `${build}/ends/${id}.json`, "--view", `bus=${view}`, "--observer", "foldiboy-sm83-views:views", "--output", "json"];
  const saved = `corpus/build/compare/${id}-${view}.json`;
  await krs("compare", args, saved);
  const output = await Bun.file(resolve(root, saved)).text();
  const result = output.trim() ? JSON.parse(output) : { verdict: "error" };
  const at = result.difference?.path ? ` at ${result.difference.path}` : "";
  console.log(`${id} ${view}: ${result.verdict}${at}`);
  return result.verdict as string;
}
const work = ids.flatMap((id) => VIEWS.map((view) => ({ id, view })));
const failed: string[] = [];
async function worker() {
  for (let next = work.shift(); next; next = work.shift()) {
    const verdict = await compare(next.id, next.view);
    if (next.view === REQUIRED && verdict !== "equal") failed.push(next.id);
  }
}
await Promise.all(Array.from({ length: JOBS }, worker));
if (failed.length) {
  console.log(`not equal under ${REQUIRED}: ${failed.join(", ")}`);
  process.exit(1);
}
