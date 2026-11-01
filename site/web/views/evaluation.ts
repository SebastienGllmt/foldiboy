// One evaluation set (a loop's epoch): candidates against metrics, the frontier, decisions and gates.
// Used on loop pages; a future evaluations route would reuse it.

import type { EvaluationSet } from "../../src/evaluations";
import type { Model } from "../../src/model";
import { h } from "../ui/dom";
import { fields, muted, small, table, tag, win } from "../ui/components";
import { markSprite, shapeSprite } from "../ui/sprites";
import { gateVerdict } from "../ui/status";
import { decision } from "./common";

/** Candidates on the frontier: no other candidate is at least as good on every metric and better on one. */
export function frontier(set: EvaluationSet): Set<string> {
  const better = (a: number, b: number, dir: string) => (dir === "lower" ? a < b : a > b);
  const front = new Set<string>();
  for (const e of set.evaluation) {
    const dominated = set.evaluation.some(
      (o) =>
        o !== e &&
        set.metric.every((x) => !better(e.metrics[x.id] ?? NaN, o.metrics[x.id] ?? NaN, x.direction)) &&
        set.metric.some((x) => better(o.metrics[x.id] ?? NaN, e.metrics[x.id] ?? NaN, x.direction)),
    );
    if (!dominated) front.add(e.id);
  }
  return front;
}

export function evaluationSet(m: Model, set: EvaluationSet) {
  const front = frontier(set);
  const best = (metric: string, dir: string) => {
    const values = set.evaluation.map((e) => e.metrics[metric]).filter((v): v is number => v !== undefined);
    return dir === "lower" ? Math.min(...values) : Math.max(...values);
  };
  const star = (v: number) => h("span", { class: "status", title: "best in this epoch" }, markSprite("star", "var(--series-4)"), String(v));
  // Candidates are columns: there are few candidates and many metrics, so this keeps rows short and comparable.
  const columns = [{ label: "Metric" }, ...set.evaluation.map((e) => ({ label: e.role === "baseline" ? `${e.id} (baseline)` : e.id }))];
  const passed = set.evaluation.flatMap((e) => e.gates).filter((g) => g.verdict === "pass").length;
  const total = set.evaluation.flatMap((e) => e.gates).length;
  return [
    win(
      `Epoch ${set.epoch.id}`,
      fields([
        ["Measures", set.epoch.definition],
        ["Conditions", set.epoch.conditions],
        ["k-rust", set.epoch.k_rust_revision ? h("code", {}, set.epoch.k_rust_revision) : undefined],
        ["Record", [h("code", {}, set.file), set.epoch.source ? [" from ", h("code", {}, set.epoch.source)] : null]],
      ]),
    ),
    win(
      "Results",
      table(columns, [
        { cells: ["Candidate", ...set.evaluation.map((e) => [e.role === "baseline" ? tag("baseline", true) : null, small(e.candidate)])] },
        { cells: ["Decision", ...set.evaluation.map((e) => decision(m, e.decision))] },
        ...set.metric.map((x) => ({
          cells: [[x.title, small(`${x.unit ?? ""}, ${x.direction} is better`)], ...set.evaluation.map((e) => {
            const v = e.metrics[x.id];
            return v === undefined ? muted("–") : v === best(x.id, x.direction) ? star(v) : String(v);
          })],
        })),
        { cells: ["Frontier", ...set.evaluation.map((e) => (front.has(e.id) ? h("span", { class: "status" }, shapeSprite("pareto", "var(--series-2)"), "on frontier") : muted("dominated")))] },
        ...(set.evaluation.some((e) => e.note) ? [{ cells: ["Note", ...set.evaluation.map((e) => (e.note ? small(e.note) : ""))] }] : []),
      ]),
      { aside: `${set.evaluation.length} candidates · ${set.metric.length} metrics` },
    ),
    win(
      "Gates",
      table(
        [{ label: "Candidate" }, { label: "Gate" }, { label: "Verdict" }, { label: "Evidence" }],
        set.evaluation.flatMap((e) => e.gates.map((g) => ({ cells: [e.id, g.gate, gateVerdict(g.verdict), h("code", {}, g.evidence)] }))),
      ),
      { aside: `${passed} of ${total} pass` },
    ),
  ];
}
