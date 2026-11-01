// Problems: what the model checks found in the tracking files.

import type { Model, Problem, Severity } from "../../src/model";
import { countBy, h } from "../ui/dom";
import { chips, dialogue, page, table, win } from "../ui/components";
import { SEVERITY_TONE, severity, status } from "../ui/status";

export const problemTable = (problems: Problem[]) =>
  table([{ label: "Severity", nowrap: true }, { label: "File", nowrap: true }, { label: "Record", nowrap: true }, { label: "Message" }], problems.map((p) => ({ cells: [severity(p.severity), h("code", {}, p.file), p.id ?? "", p.message] })));

export function problemsView(m: Model) {
  const order: Severity[] = ["error", "warning", "info"];
  const sorted = [...m.problems].sort((a, b) => order.indexOf(a.severity) - order.indexOf(b.severity));
  return page(
    { title: "Problems", sub: `${m.problems.length} found` },
    sorted.length
      ? [win("By severity", chips(countBy(sorted, (p) => p.severity).map(([sev, n]) => status(SEVERITY_TONE[sev as Severity], `${sev} ${n}`)))), win("Check results", problemTable(sorted))]
      : dialogue("The tracking files pass every check.", "good"),
  );
}
