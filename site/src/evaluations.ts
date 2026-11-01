// Loads authored evaluation records: one TOML file per loop epoch under evaluations/.

import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import type { Problem } from "./model";

export interface Metric {
  id: string;
  title: string;
  unit?: string;
  direction: "lower" | "higher";
}

export interface Gate {
  gate: string;
  verdict: "pass" | "fail" | "open";
  evidence: string;
}

export interface Evaluation {
  id: string;
  candidate: string;
  role: "baseline" | "candidate";
  decision: string;
  note?: string;
  metrics: Record<string, number>;
  gates: Gate[];
}

export interface EvaluationSet {
  file: string;
  loop: string;
  epoch: { id: string; definition: string; k_rust_revision?: string; conditions?: string; source?: string };
  metric: Metric[];
  evaluation: Evaluation[];
}

export async function loadEvaluations(dir: string, known: { loops: Set<string>; decisions: Set<string>; evidence: Set<string> }, problems: Problem[]): Promise<EvaluationSet[]> {
  const evalDir = resolve(dir, "evaluations");
  const files = (await readdir(evalDir).catch(() => [] as string[])).filter((f) => f.endsWith(".toml")).sort();
  const sets: EvaluationSet[] = [];
  for (const name of files) {
    const file = `evaluations/${name}`;
    let set: EvaluationSet;
    try {
      set = { file, ...(Bun.TOML.parse(await Bun.file(resolve(evalDir, name)).text()) as Omit<EvaluationSet, "file">) };
    } catch (error) {
      problems.push({ severity: "error", file, message: `cannot load: ${(error as Error).message}` });
      continue;
    }
    set.metric ??= [];
    set.evaluation ??= [];
    if (!known.loops.has(set.loop)) problems.push({ severity: "error", file, message: `unknown loop '${set.loop}'` });
    if (!set.epoch?.id) problems.push({ severity: "error", file, message: "missing [epoch] id" });
    const metricIds = new Set(set.metric.map((m) => m.id));
    if (!set.evaluation.some((e) => e.role === "baseline")) problems.push({ severity: "warning", file, message: "epoch has no baseline evaluation" });
    for (const e of set.evaluation) {
      if (!known.decisions.has(e.decision)) problems.push({ severity: "error", file, id: e.id, message: `unknown decision '${e.decision}'` });
      for (const m of Object.keys(e.metrics ?? {})) if (!metricIds.has(m)) problems.push({ severity: "error", file, id: e.id, message: `unknown metric '${m}'` });
      for (const g of e.gates ?? []) if (!known.evidence.has(g.evidence)) problems.push({ severity: "error", file, id: e.id, message: `unknown evidence level '${g.evidence}'` });
    }
    sets.push(set);
  }
  return sets;
}
