// Derives coverage universes and results from existing project artifacts.
// Every dataset carries its provenance; a missing source is reported, not invented.

import { realpathSync } from "node:fs";
import { readdir, stat } from "node:fs/promises";
import { dirname, relative, resolve } from "node:path";
import type { Problem } from "./model";

export interface Provenance {
  path: string;
  modified?: string;
  revision?: string;
}

export interface Dataset<T> {
  id: string;
  title: string;
  provenance: Provenance[];
  rows: T[];
}

export interface OpcodeSlot {
  page: "base" | "cb";
  code: number;
  mnemonic: string;
  group: string;
  vectorsPassed?: number;
  vectorsTotal?: number;
  failure?: string;
}

export interface HardwareRegister {
  address: string;
  name: string;
  description: string;
  access: string;
  models: string;
}

export interface MemoryRegion {
  start: string;
  end: string;
  description: string;
  notes: string;
}

export interface SuiteTest {
  suite: string;
  test: string;
}

export interface StateCell {
  cell: string;
  initial: string;
}

export interface ClaimResult {
  claim: string;
  module: string;
  verdict: string;
  modality: string;
  source: string;
}

// Keep the logical path, as scripts/test.sh does: the workspace may reach this repository through a symlink,
// and docs/ and roms/ sit beside the logical checkout, not beside the canonical clone.
function logicalRepoRoot(): string {
  const physical = resolve(import.meta.dir, "../..");
  const real = (path: string) => {
    try {
      return realpathSync(path);
    } catch {
      return undefined;
    }
  };
  for (let dir = process.env.PWD; dir && dir !== dirname(dir); dir = dirname(dir)) {
    if (real(dir) === real(physical)) return dir;
  }
  return physical;
}

export const repoRoot = logicalRepoRoot();
export const workspaceRoot = resolve(process.env.FOLDIBOY_WORKSPACE ?? resolve(repoRoot, ".."));
const testRoot = resolve(process.env.FOLDIBOY_TEST_ROOT ?? resolve(workspaceRoot, "roms/tests"));
const sm83 = resolve(repoRoot, "krs/sm83");

const display = (path: string) => (path.startsWith(repoRoot) ? relative(repoRoot, path) : path.replace(workspaceRoot, "<workspace>"));

async function provenance(path: string, gitDir?: string): Promise<Provenance> {
  const info = await stat(path).catch(() => undefined);
  let revision: string | undefined;
  if (gitDir) {
    const proc = Bun.spawnSync(["git", "-C", gitDir, "rev-parse", "--short", "HEAD"]);
    if (proc.exitCode === 0) revision = proc.stdout.toString().trim();
  }
  return { path: display(path), modified: info?.mtime.toISOString(), revision };
}

async function guard<T>(id: string, problems: Problem[], load: () => Promise<Dataset<T>>): Promise<Dataset<T> | undefined> {
  try {
    return await load();
  } catch (error) {
    problems.push({ severity: "warning", file: `source:${id}`, message: `dataset unavailable: ${(error as Error).message}` });
    return undefined;
  }
}

// Rows of a Markdown pipe table that follows the given header line.
function markdownTable(text: string, header: RegExp): string[][] {
  const lines = text.split("\n");
  const start = lines.findIndex((line) => header.test(line));
  if (start < 0) throw new Error(`table header ${header} not found`);
  const rows: string[][] = [];
  for (const line of lines.slice(start + 2)) {
    if (!line.includes("|")) break;
    rows.push(line.split("|").map((cell) => cell.trim()));
  }
  return rows;
}

const stripLinks = (s: string) => s.replace(/\[([^\]]*)\](\([^)]*\))?/g, "$1").replace(/[<>]/g, "");

export async function loadOpcodes(problems: Problem[]) {
  return guard("opcodes", problems, async () => {
    const opsPath = resolve(workspaceRoot, "docs/dmgops.json");
    const vectorsPath = resolve(sm83, "results/vectors.json");
    const ops = (await Bun.file(opsPath).json()) as Record<"Unprefixed" | "CBPrefixed", { Name: string; Group: string }[]>;
    const vectors = (await Bun.file(vectorsPath).json()) as { files: { file: string; passed: number; total: number; failures: { reason: string }[] }[] };
    const byFile = new Map(vectors.files.map((f) => [f.file, f]));
    const rows: OpcodeSlot[] = [];
    for (const [page, key, prefix] of [["base", "Unprefixed", ""], ["cb", "CBPrefixed", "cb "]] as const) {
      ops[key].forEach((op, code) => {
        const file = byFile.get(`${prefix}${code.toString(16).padStart(2, "0")}`);
        rows.push({
          page,
          code,
          mnemonic: op.Name,
          group: op.Group,
          vectorsPassed: file?.passed,
          vectorsTotal: file?.total,
          failure: file?.failures[0]?.reason,
        });
      });
    }
    return { id: "opcodes", title: "Opcode slots", provenance: [await provenance(opsPath), await provenance(vectorsPath)], rows };
  });
}

export async function loadHardwareRegisters(problems: Problem[]) {
  return guard("registers", problems, async () => {
    const path = resolve(workspaceRoot, "docs/pandocs/src/Hardware_Reg_List.md");
    const rows = markdownTable(await Bun.file(path).text(), /^Address\s*\|\s*Name/).map(([address = "", name = "", description = "", access = "", models = ""]) => ({
      address,
      name: stripLinks(name),
      description,
      access,
      models,
    }));
    return { id: "registers", title: "Hardware registers (Pan Docs)", provenance: [await provenance(path, resolve(workspaceRoot, "docs/pandocs"))], rows };
  });
}

export async function loadMemoryMap(problems: Problem[]) {
  return guard("memory", problems, async () => {
    const path = resolve(workspaceRoot, "docs/pandocs/src/Memory_Map.md");
    const rows = markdownTable(await Bun.file(path).text(), /^Start\s*\|\s*End\s*\|\s*Description/).map(([start = "", end = "", description = "", notes = ""]) => ({
      start: `$${start}`,
      end: `$${end}`,
      description: stripLinks(description),
      notes: stripLinks(notes),
    }));
    return { id: "memory", title: "Memory map (Pan Docs)", provenance: [await provenance(path, resolve(workspaceRoot, "docs/pandocs"))], rows };
  });
}

export async function loadMooneye(problems: Problem[]) {
  return guard("mooneye", problems, async () => {
    const dir = resolve(testRoot, "mooneye-test-suite/acceptance");
    const files = (await readdir(dir, { recursive: true })).filter((f) => f.endsWith(".s")).sort();
    const rows = files.map((f) => ({ suite: "mooneye acceptance", test: f.replace(/\.s$/, "") }));
    return { id: "mooneye", title: "Mooneye acceptance tests", provenance: [await provenance(dir, resolve(testRoot, "mooneye-test-suite"))], rows };
  });
}

export async function loadStateCells(problems: Problem[]) {
  return guard("state", problems, async () => {
    const path = resolve(sm83, "results/emitted/definition.k");
    const text = await Bun.file(path).text();
    const block = text.slice(text.indexOf("configuration"), text.indexOf("rule ", text.indexOf("configuration")));
    const rows = [...block.matchAll(/<(\w+)>\s*(.*?)\s*<\/\1>/g)].map(([, cell = "", initial = ""]) => ({ cell, initial }));
    if (!rows.length) throw new Error("no configuration cells found");
    return { id: "state", title: "Reference state cells", provenance: [await provenance(path)], rows };
  });
}

export async function loadClaims(problems: Problem[]) {
  return guard("claims", problems, async () => {
    const path = resolve(sm83, "results/claims.json");
    const doc = (await Bun.file(path).json()) as { prove: { claim: string; verdict: string; modality: string; origin?: { file: string; line: number } }[] };
    const rows = doc.prove.map((p) => ({
      claim: p.claim.split(".").slice(1).join("."),
      module: p.claim.split(".")[0] ?? "",
      verdict: p.verdict,
      modality: p.modality,
      source: p.origin ? `${p.origin.file}:${p.origin.line}` : "",
    }));
    return { id: "claims", title: "Native claim results", provenance: [await provenance(path)], rows };
  });
}
