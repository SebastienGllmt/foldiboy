// Project proof workflow: replays the native contract suite and records a machine-readable result.
//
//   bun run-claims.ts            all claims of SM83-CONTRACTS-SPEC
//
// The claim set is read from the emitted spec.k. Every claim is proved by one `krs prove --output jsonl`, which proves
// each claim in a supervised worker and prints its record as it ends: a claim whose proof runs longer than
// CLAIM_TIMEOUT seconds (default 600) gets the verdict `timeout`, a worker crash costs only its claim (`crashed`), and a
// run cut short keeps the verdicts it reached. Solver queries are bounded by Z3 resource units, not the clock, so a
// verdict does not depend on the host's load. A claim with no record is listed as missing.
//
// Writes results/claims.json (the krs-exec/2 records of every claim) and results/claims-run.json (checkout
// revisions, the krs binary's modification time and digest, the solver, and the invocation's command, times and exit
// status).

import { mkdir, rm } from "node:fs/promises";
import { resolve } from "node:path";

const here = import.meta.dir;
const env = process.env;
const KRS_ROOT = env.KRS_ROOT ?? "/artifacts/worktrees/k-rust-foldiboy-pin";
const KRS_BIN = env.KRS_BIN ?? `${KRS_ROOT}/target/debug/krs`;
const SPEC_MODULE = env.SPEC_MODULE ?? "SM83-CONTRACTS-SPEC";
const DEPTH = env.DEPTH ?? "32";
const CLAIM_TIMEOUT = env.CLAIM_TIMEOUT ?? "600";

const results = resolve(here, "results");
const childEnv = {
  ...env,
  CARGO_TARGET_DIR: resolve(here, "target"),
};
const base = ["--manifest-path", "Cargo.toml", "--package", "foldiboy-sm83", "--definition", "main", "--dependency", "foldiboy-sm83-core:core", "--dependency", "foldiboy-sm83-events:events"];

function git(dir: string) {
  const rev = Bun.spawnSync(["git", "-C", dir, "rev-parse", "HEAD"]).stdout.toString().trim();
  const dirty = Bun.spawnSync(["git", "-C", dir, "status", "--porcelain"]).stdout.toString().trim() !== "";
  return { revision: rev, dirty };
}

// Every claim label of the emitted spec, with the module that declares it.
async function claimSet(): Promise<Map<string, string>> {
  const out = resolve(results, "claims-emit");
  await rm(out, { recursive: true, force: true });
  const proc = Bun.spawnSync([KRS_BIN, "emit-k", ...base, "--output", out], { cwd: here, env: childEnv, stderr: "inherit" });
  if (proc.exitCode !== 0) throw new Error(`krs emit-k exited with ${proc.exitCode}`);
  const labels = new Map<string, string>();
  let module = "";
  for (const line of (await Bun.file(resolve(out, "spec.k")).text()).split("\n")) {
    const m = line.match(/^module\s+(\S+)/);
    if (m) module = m[1]!;
    const c = line.match(/^\s*claim \[([^\]]+)\]/);
    if (c) labels.set(c[1]!, module);
  }
  return labels;
}

interface Invocation {
  command: string[];
  started: string;
  finished: string;
  exit_status: number | null;
  /** Whether the last line was the summary: a run without one did not finish. */
  finished_run: boolean;
}

interface ProveRecord {
  claim: string;
  verdict: string;
  [key: string]: unknown;
}

async function prove(): Promise<{ invocation: Invocation; records: ProveRecord[]; solver: unknown }> {
  const command = [KRS_BIN, "prove", ...base, "--spec-module", SPEC_MODULE, "--depth", DEPTH, "--claim-timeout", CLAIM_TIMEOUT, "--output", "jsonl"];
  const started = new Date();
  console.error(`${started.toISOString()} proving every claim of ${SPEC_MODULE} (claim timeout ${CLAIM_TIMEOUT} s)`);
  const proc = Bun.spawn(command, { cwd: here, env: childEnv, stdout: "pipe", stderr: "inherit" });
  const stdout = await new Response(proc.stdout).text();
  const exit = await proc.exited;
  const finished = new Date();
  const lines = stdout.split("\n").filter((line) => line.trim()).map((line) => JSON.parse(line) as { schema: string; proof?: ProveRecord; summary?: unknown; solver?: unknown });
  const records = lines.flatMap((line) => (line.proof ? [line.proof] : []));
  const summary = lines.at(-1)?.summary !== undefined ? lines.at(-1)! : undefined;
  console.error(`${finished.toISOString()} exit ${exit}, ${records.length} records${summary ? "" : " (the run did not finish)"}`);
  return {
    invocation: { command, started: started.toISOString(), finished: finished.toISOString(), exit_status: proc.signalCode ? null : exit, finished_run: summary !== undefined },
    records,
    solver: summary?.solver ?? null,
  };
}

await mkdir(results, { recursive: true });
const krs = git(KRS_ROOT);
// The CLI may predate the checkout's HEAD; record the binary itself so the two can be compared.
const binary = { path: KRS_BIN, modified: (await Bun.file(KRS_BIN).stat()).mtime.toISOString(), sha256: new Bun.CryptoHasher("sha256").update(await Bun.file(KRS_BIN).arrayBuffer()).digest("hex") };
const foldiboy = git(here);
const labels = await claimSet();
const all = labels.size;
const { invocation, records, solver } = await prove();

const reported = new Set(records.map((r) => r.claim.split(".").slice(1).join(".")));
const missing = [...labels.keys()].filter((l) => !reported.has(l));
await Bun.write(resolve(results, "claims.json"), JSON.stringify({ schema: "krs-exec/2", prove: records }, null, 2) + "\n");
await Bun.write(
  resolve(results, "claims-run.json"),
  JSON.stringify({ schema: "foldiboy-claims-run/3", spec_module: SPEC_MODULE, depth: Number(DEPTH), claim_timeout_seconds: Number(CLAIM_TIMEOUT), k_rust: krs, krs_binary: binary, foldiboy, solver, claim_count: all, missing, invocation }, null, 2) + "\n",
);
const counts = Object.entries(Object.groupBy(records, (r) => r.verdict)).map(([v, rs]) => `${rs!.length} ${v}`);
console.error(`${records.length} of ${all} claims recorded: ${counts.join(", ")}${missing.length ? `; missing ${missing.join(", ")}` : ""}`);
