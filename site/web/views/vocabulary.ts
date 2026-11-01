// Vocabulary: every controlled term, with the way the site draws it.

import type { Model, Term } from "../../src/model";
import { h, join } from "../ui/dom";
import { page, table, win } from "../ui/components";
import { shapeSprite } from "../ui/sprites";
import { statusOf } from "../ui/status";
import { loopLink } from "./common";

const statusList = (title: string, terms: Term[]) => win(title, table([{ label: "Status", nowrap: true }, { label: "Tone" }, { label: "Meaning" }], terms.map((t) => ({ cells: [statusOf(terms, t.id), String(t.tone ?? "neutral"), t.meaning ?? ""] }))));

export function vocabularyView(m: Model) {
  const v = m.vocabulary;
  const usedBy = (shape: string) => m.loops.filter((l) => l.shape.includes(shape)).map((l) => loopLink(l.id));
  return page(
    { title: "Vocabulary", sub: "vocabulary.toml" },
    win(
      "Shapes",
      table(
        [{ label: "Shape", nowrap: true }, { label: "Progress means" }, { label: "Examples" }, { label: "Planned views" }, { label: "Loops" }],
        v.shape.map((t) => ({ cells: [h("span", { class: "status" }, shapeSprite(t.id), t.id), t.progress ?? "", t.examples ?? "", join(t.views ?? [], "; "), join(usedBy(t.id))] })),
      ),
    ),
    win("Evidence levels", table([{ label: "#", num: true }, { label: "Level", nowrap: true }, { label: "Meaning" }], v.evidence_level.map((t, i) => ({ cells: [i + 1, h("code", {}, t.id), t.meaning ?? ""] }))), { aside: "provisional order, strongest first" }),
    win("Loop maturity", table([{ label: "Maturity", nowrap: true }, { label: "Meaning" }], v.maturity.map((t) => ({ cells: [h("span", { class: "tag" }, t.id), t.meaning ?? ""] })))),
    statusList("Ticket status", v.ticket_status),
    statusList("Capability status", v.capability_status),
    statusList("Coverage status", v.coverage_status),
    statusList("Evaluation decision", v.decision),
  );
}
