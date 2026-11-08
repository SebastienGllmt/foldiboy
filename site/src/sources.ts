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

// Keep the logical path, as scripts/roms.ts does: the workspace may reach this repository through a symlink,
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

// The records of a JSON-lines file krs wrote (bun run sst, mooneye, claims).
async function jsonLines<T>(path: string): Promise<T[]> {
  return (await Bun.file(path).text()).split("\n").filter((line) => line.trim()).map((line) => JSON.parse(line) as T);
}

// A `krs cases` record: a case's verdict, or the run's summary.
interface CaseRecord {
  case?: string;
  verdict?: string;
  reason?: string;
  difference?: { path?: string };
}
const caseName = (record: CaseRecord) => record.case!.split("#").at(-1)!;
const caseFailure = (record: CaseRecord) => record.reason ?? (record.difference?.path ? `differs at ${record.difference.path}` : record.verdict);

async function guard<D extends Dataset<unknown>>(id: string, problems: Problem[], load: () => Promise<D>): Promise<D | undefined> {
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
    const vectorsPath = resolve(repoRoot, "krs/oracles/singlestep/results/sm83.jsonl");
    const ops = (await Bun.file(opsPath).json()) as Record<"Unprefixed" | "CBPrefixed", { Name: string; Group: string }[]>;
    // Per suite file (a case document named as the suite file): vectors, met, and the first failure.
    const byFile = new Map<string, { passed: number; total: number; failure?: string }>();
    for (const record of await jsonLines<CaseRecord>(vectorsPath)) {
      if (!record.case) continue;
      const name = record.case.split("#")[0]!.split("/").at(-1)!.replace(/\.json$/, "");
      const file = byFile.get(name) ?? { passed: 0, total: 0 };
      file.total += 1;
      if (record.verdict === "met") file.passed += 1;
      else file.failure ??= caseFailure(record);
      byFile.set(name, file);
    }
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
          failure: file?.failure,
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
    const reportPath = resolve(repoRoot, "krs/oracles/mooneye/results/sm83.jsonl");
    const report = (await Bun.file(reportPath).exists()) ? (await jsonLines<CaseRecord>(reportPath)).filter((r) => r.case) : undefined;
    // A case is named as its test's file, without the directory (krs/oracles/mooneye/sm83.toml).
    const results = new Map(report?.map((r) => [caseName(r), r]));
    const rows = files.map((f) => {
      const test = f.replace(/\.s$/, "");
      const result = results.get(test.split("/").at(-1)!);
      return { suite: "mooneye acceptance", test, verdict: result?.verdict, reason: result && result.verdict !== "met" ? caseFailure(result) : undefined };
    });
    const provenances = [await provenance(dir, resolve(testRoot, "mooneye-test-suite"))];
    if (report) provenances.push(await provenance(reportPath));
    return { id: "mooneye", title: "Mooneye acceptance tests", provenance: provenances, rows, reported: report !== undefined };
  });
}

export async function loadStateCells(problems: Problem[]) {
  return guard("state", problems, async () => {
    const path = resolve(repoRoot, "corpus/definition.k");
    const text = await Bun.file(path).text();
    const block = text.slice(text.indexOf("configuration"), text.indexOf("rule ", text.indexOf("configuration")));
    const rows = [...block.matchAll(/<(\w+)>\s*(.*?)\s*<\/\1>/g)].map(([, cell = "", initial = ""]) => ({ cell, initial }));
    if (!rows.length) throw new Error("no configuration cells found");
    return { id: "state", title: "Reference state cells", provenance: [await provenance(path)], rows };
  });
}

export async function loadClaims(problems: Problem[]) {
  return guard("claims", problems, async () => {
    // `krs prove --output jsonl` (bun run claims): a record per claim, then the run's summary.
    const path = resolve(sm83, "results/claims.jsonl");
    const lines = await jsonLines<{ proof?: { claim: string; verdict: string; modality: string; origin?: { file: string; line: number } }; summary?: unknown }>(path);
    const rows = lines.flatMap(({ proof: p }) =>
      p
        ? [{
            claim: p.claim.split(".").slice(1).join("."),
            module: p.claim.split(".")[0] ?? "",
            verdict: p.verdict,
            modality: p.modality,
            source: p.origin ? `${p.origin.file}:${p.origin.line}` : "",
          }]
        : [],
    );
    if (lines.at(-1)?.summary === undefined) problems.push({ severity: "warning", file: "source:claims", message: "the recorded claims run did not finish (no summary line)" });
    return { id: "claims", title: "Native claim results", provenance: [await provenance(path)], rows };
  });
}
