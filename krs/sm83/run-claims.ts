// Project proof workflow: replays the native contract suite under time budgets and records a machine-readable result.
//
//   bun run-claims.ts            all claims of SM83-CONTRACTS-SPEC
//
// The claim set is read from the emitted spec.k. Claims listed in SLOW_CLAIMS run one at a time under SLOW_BUDGET seconds;
// the rest run as one batch under BATCH_BUDGET seconds. A claim whose invocation exceeds its budget is recorded with
// verdict "budget-exceeded", which establishes nothing about the claim.
//
// Writes results/claims.json (the krs prove JSON records of every invocation, merged) and results/claims-run.json
// (checkout revisions, the krs binary's modification time and digest, budgets, and each invocation's command, times and exit status).

import { mkdir, rm } from "node:fs/promises";
import { resolve } from "node:path";

const here = import.meta.dir;
const env = process.env;
const KRS_ROOT = env.KRS_ROOT ?? "/home/sebastiengllmt/git/workspaces/starstream-fv/inspirations/midnightntwrk/k-rust";
const KRS_BIN = env.KRS_BIN ?? `${KRS_ROOT}/target/debug/krs`;
const Z3_LIB = env.Z3_LIB ?? "/nix/store/bncr69nhmgvphbynb1a3g4a2jb9b8vj0-z3-4.12.5-lib/lib";
const SPEC_MODULE = env.SPEC_MODULE ?? "SM83-CONTRACTS-SPEC";
const DEPTH = env.DEPTH ?? "32";
// The three contracts that reached the 90-second limit in the last full replay (src/claims/README.md, "Proof evidence").
const SLOW_CLAIMS = (env.SLOW_CLAIMS ?? "contract-call-taken contract-dispatch-running contract-dispatch-wake").split(/\s+/).filter(Boolean);
const SLOW_BUDGET = Number(env.SLOW_BUDGET ?? 600);
const BATCH_BUDGET = Number(env.BATCH_BUDGET ?? 3600);

const results = resolve(here, "results");
const childEnv = {
  ...env,
  CARGO_TARGET_DIR: resolve(here, "target"),
  LD_LIBRARY_PATH: [Z3_LIB, env.LD_LIBRARY_PATH].filter(Boolean).join(":"),
  LIBRARY_PATH: [Z3_LIB, env.LIBRARY_PATH].filter(Boolean).join(":"),
};
const base = ["--manifest-path", "Cargo.toml", "--package", "foldiboy-sm83", "--definition", "main", "--dependency", "foldiboy-sm83-core:core"];

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
  claims: string[];
  budget_seconds: number;
  command: string[];
  started: string;
  finished: string;
  exit_status: number | null;
  budget_exceeded: boolean;
}

interface ProveRecord {
  claim: string;
  verdict: string;
  [key: string]: unknown;
}

async function prove(claims: string[], budget: number, all: number): Promise<{ invocation: Invocation; records: ProveRecord[] }> {
  const selection = claims.length === all ? [] : claims.flatMap((c) => ["--claim", c]);
  const command = [KRS_BIN, "prove", ...base, "--spec-module", SPEC_MODULE, "--depth", DEPTH, "--output", "json", ...selection];
  const started = new Date();
  console.error(`${started.toISOString()} proving ${claims.length === 1 ? claims[0] : `${claims.length} claims`} (budget ${budget} s)`);
  const proc = Bun.spawn(command, { cwd: here, env: childEnv, stdout: "pipe", stderr: "inherit", timeout: budget * 1000, killSignal: "SIGKILL" });
  const stdout = await new Response(proc.stdout).text();
  const exit = await proc.exited;
  const finished = new Date();
  const budget_exceeded = finished.getTime() - started.getTime() >= budget * 1000 - 100 && proc.signalCode !== null;
  let records: ProveRecord[] = [];
  if (!budget_exceeded && stdout.trim()) records = (JSON.parse(stdout) as { prove: ProveRecord[] }).prove;
  console.error(`${finished.toISOString()} exit ${exit}${budget_exceeded ? " (budget exceeded)" : ""}, ${records.length} records`);
  return {
    invocation: { claims, budget_seconds: budget, command, started: started.toISOString(), finished: finished.toISOString(), exit_status: proc.signalCode ? null : exit, budget_exceeded },
    records,
  };
}

await mkdir(results, { recursive: true });
const krs = git(KRS_ROOT);
// The CLI may predate the checkout's HEAD; record the binary itself so the two can be compared.
const binary = { path: KRS_BIN, modified: (await Bun.file(KRS_BIN).stat()).mtime.toISOString(), sha256: new Bun.CryptoHasher("sha256").update(await Bun.file(KRS_BIN).arrayBuffer()).digest("hex") };
const foldiboy = git(here);
const labels = await claimSet();
for (const slow of SLOW_CLAIMS) if (!labels.has(slow)) throw new Error(`slow claim ${slow} is not in spec.k`);
const all = labels.size;
const batch = [...labels.keys()].filter((l) => !SLOW_CLAIMS.includes(l));

const invocations: Invocation[] = [];
const records: ProveRecord[] = [];
for (const [claims, budget] of [[batch, BATCH_BUDGET] as const, ...SLOW_CLAIMS.map((c) => [[c], SLOW_BUDGET] as const)]) {
  const run = await prove([...claims], budget, all);
  invocations.push(run.invocation);
  records.push(...run.records);
  if (run.invocation.budget_exceeded) {
    for (const c of claims) records.push({ claim: `${labels.get(c)}.${c}`, verdict: "budget-exceeded", budget_seconds: budget });
  }
}

const reported = new Set(records.map((r) => r.claim.split(".").slice(1).join(".")));
const missing = [...labels.keys()].filter((l) => !reported.has(l));
await Bun.write(resolve(results, "claims.json"), JSON.stringify({ schema: "krs-exec/1", prove: records }, null, 2) + "\n");
await Bun.write(
  resolve(results, "claims-run.json"),
  JSON.stringify({ schema: "foldiboy-claims-run/2", spec_module: SPEC_MODULE, depth: Number(DEPTH), k_rust: krs, krs_binary: binary, foldiboy, claim_count: all, missing, invocations }, null, 2) + "\n",
);
const counts = Object.entries(Object.groupBy(records, (r) => r.verdict)).map(([v, rs]) => `${rs!.length} ${v}`);
console.error(`${records.length} of ${all} claims recorded: ${counts.join(", ")}${missing.length ? `; missing ${missing.join(", ")}` : ""}`);
