// Runs the SingleStepTests SM83 suite (v1) on the definition:
//
//   bun scripts/singlestep.ts [--wz W,Z] [--jobs N] [FILE...]     (FILE: a suite file's name, e.g. 00 "cb 40"; all by default)
//
// Converts each suite file into a krs case document (krs/sm83/results/singlestep-cases/), runs
// them with `krs cases`, and saves its records to krs/sm83/results/singlestep.jsonl.
//
// Each vector is one instruction from a boundary state: a case that starts from the vector's
// initial state and expects its final state and bus cycles. What the suite does not state:
// - The suite treats all 64 KiB as RAM, so the ROM is empty and `memory` holds the listed
//   bytes. `IE` ($FFFF) and `IF` ($FF0F), which every boundary reads, are zero where a vector
//   does not list them: no interrupt is requested.
// - `F` is a byte in the suite and four flags in the definition (bits 7 to 4).
// - `EI`'s pending state, `HALT` mode and the `HALT` bug start clear. `W` and `Z` are
//   unobservable, so their initial value is a choice (--wz, default 0,0); every vector must give
//   the same verdict under any value.
// - An idle cycle is compared by kind: the suite records bus lines the definition does not model.
// - The expected memory is the initial memory with the final bytes written over it.
// - A step runs to the next boundary rule: stop before one, count 2 (the first stop is the
//   initial state). The next boundary fetches, except after `HALT`. The suite models `HALT`
//   ($76) and `STOP` ($10) as one-cycle no-ops; the definition does not, so those 2000 vectors fail.

import { mkdir, readdir, rm } from "node:fs/promises";
import { resolve } from "node:path";
import { krs, printSummary, root } from "./krs";
import { checkout } from "./roms";

const REGISTERS = ["a", "b", "c", "d", "e", "h", "l"];
const IE = 0xffff;
const IF = 0xff0f;
const BOUNDARY = ["dispatch", "fetch", "wake", "halted"].map((rule) => `crate::interrupt::boundary_${rule}`);
const COMPARED = ["regs", "f", "pc", "sp", "ime", "ei_pending", "memory", "bus"];

interface State { a: number; b: number; c: number; d: number; e: number; h: number; l: number; f: number; pc: number; sp: number; ime: number; ei?: number; ram: [number, number][] }
interface Vector { name: string; initial: State; final: State; cycles: [number, number, string][] }

const flags = (byte: number) => ({ Flags: { z: !!(byte & 0x80), n: !!(byte & 0x40), h: !!(byte & 0x20), c: !!(byte & 0x10) } });
const registers = (state: State) => Object.fromEntries(REGISTERS.map((r) => [r.toUpperCase(), state[r as keyof State]]));
// Inside a State, krs does not carry the register map's key type, so its keys are spelled out.
const startRegisters = (state: State) => ({ $map: REGISTERS.map((r) => [{ [r.toUpperCase()]: [] }, state[r as keyof State]]) });
const memory = (bytes: Map<number, number>) => ({ $map: [...bytes].sort(([a], [b]) => a - b) });

function event([addr, value, kind]: [number, number, string]) {
  if (kind === "r-m") return { Read: { addr, value } };
  if (kind === "-wm") return { Write: { addr, value } };
  if (kind === "---") return "Idle";
  throw new Error(`unknown cycle kind ${kind}`);
}

function toCase(vector: Vector, wz: [number, number]) {
  const { initial, final } = vector;
  const ram = new Map(initial.ram);
  for (const address of [IE, IF]) if (!ram.has(address)) ram.set(address, 0);
  const expected = new Map([...ram, ...final.ram]);
  const haltEntered = ram.get(initial.pc) === 0x76;
  return {
    name: vector.name,
    inputs: {
      start: {
        State: {
          regs: startRegisters(initial), f: flags(initial.f), pc: initial.pc, sp: initial.sp, ime: initial.ime !== 0,
          ei_pending: false, halted: false, halt_bug: false, w: wz[0], z: wz[1], rom: { $hex: "" }, memory: memory(ram),
        },
      },
    },
    expect: {
      halt: "stopped",
      rule: haltEntered ? BOUNDARY[3] : BOUNDARY[1],
      compare: COMPARED,
      cells: {
        regs: registers(final), f: flags(final.f), pc: final.pc, sp: final.sp, ime: final.ime !== 0,
        ei_pending: (final.ei ?? 0) !== 0, memory: memory(expected), bus: vector.cycles.map(event),
      },
    },
  };
}

const args = Bun.argv.slice(2);
const option = (name: string, fallback: string) => {
  const at = args.indexOf(name);
  return at >= 0 ? args.splice(at, 2)[1]! : fallback;
};
const wz = option("--wz", "0,0").split(",").map(Number) as [number, number];
const jobs = option("--jobs", "8");

const suite = resolve(checkout("sm83"), "v1");
const out = resolve(root, "krs/sm83/results/singlestep-cases");
const files = args.length ? args : (await readdir(suite)).filter((f) => f.endsWith(".json")).map((f) => f.slice(0, -5)).sort();
await rm(out, { recursive: true, force: true });
await mkdir(out, { recursive: true });
for (const file of files) {
  const vectors = (await Bun.file(`${suite}/${file}.json`).json()) as Vector[];
  const document = { run: { stop_before: BOUNDARY, stop_count: 2, depth: 64 }, case: vectors.map((v) => toCase(v, wz)) };
  await Bun.write(`${out}/${file}.json`, JSON.stringify(document) + "\n");
}
console.error(`converted ${files.length} suite files (W,Z = ${wz.join(",")})`);

const documents = files.map((file) => `${out}/${file}.json`);
const records = "krs/sm83/results/singlestep.jsonl";
const status = await krs("cases", [...documents, "--jobs", jobs, "--output", "json"], records);
await printSummary(records);
process.exit(status);
