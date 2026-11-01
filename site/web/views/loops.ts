// Loops: every loop by area, and one page per loop.

import type { Loop, Model } from "../../src/model";
import { countBy, h, join } from "../ui/dom";
import { chips, empty, fields, list, page, route, small, table, win } from "../ui/components";
import { shapeSprite } from "../ui/sprites";
import { termOf } from "../ui/status";
import { capabilityStatus, capLink, fileStatusNotice, loopLink, ticketLink } from "./common";
import { evaluationSet } from "./evaluation";
import { problemTable } from "./problems";

const capsOf = (m: Model, l: Loop) => (m.derived.loopCapabilities[l.id] ?? []).map((id) => m.capabilities.find((c) => c.id === id)!).filter(Boolean);
const shapes = (l: Loop) => h("span", { class: "status" }, l.shape.map((x) => shapeSprite(x)), l.shape.join(", "));
const maturity = (m: Model, l: Loop) => h("span", { class: "tag", title: termOf(m.vocabulary.maturity, l.maturity).meaning }, l.maturity);

export function loopsView(m: Model) {
  const areas = [...new Set(m.loops.map((l) => l.area))];
  return page(
    { title: "Loops", sub: `${m.loops.length} loops in ${areas.length} areas` },
    fileStatusNotice(m, ["loops.toml"]),
    areas.map((area) => {
      const loops = m.loops.filter((l) => l.area === area);
      return win(
        area,
        table(
          [{ label: "Loop", nowrap: true }, { label: "Maturity" }, { label: "Shape" }, { label: "Objective" }, { label: "Blocked by" }, { label: "k-rust" }],
          loops.map((l) => ({
            id: l.id,
            href: route("loops", l.id),
            cells: [
              [loopLink(l.id), m.evaluations.some((e) => e.loop === l.id) ? small("has evaluations") : null],
              maturity(m, l),
              shapes(l),
              l.objective,
              join((l.blocked_by ?? []).map(ticketLink)),
              chips(countBy(capsOf(m, l), (c) => c.status).map(([st, n]) => capabilityStatus(m, st, `${st} ${n}`))),
            ],
          })),
        ),
        { aside: `${loops.length} loops` },
      );
    }),
    win(
      "Gaps",
      table([{ label: "Gap", nowrap: true }, { label: "Layer" }, { label: "Description" }], m.gaps.map((g) => ({ id: g.id, cells: [g.id, g.layer, g.description] }))),
      { aside: "no loop can close these yet" },
    ),
  );
}

export function loopDetail(m: Model, id: string) {
  const l = m.loops.find((x) => x.id === id);
  const crumb = { href: route("loops"), label: "Loops" };
  if (!l) return page({ title: "Unknown loop", crumb }, empty(`No loop is called ${id}.`));
  const caps = capsOf(m, l);
  const views = l.shape.flatMap((x) => m.vocabulary.shape.find((t) => t.id === x)?.views ?? []);
  const sets = m.evaluations.filter((e) => e.loop === l.id);
  const problems = m.problems.filter((p) => p.id === l.id);
  return page(
    { title: l.id, sub: l.area, crumb },
    win(
      "Definition",
      fields([
        ["Maturity", maturity(m, l)],
        ["Shape", shapes(l)],
        ["Objective", l.objective],
        ["Candidates", l.candidates],
        ["Gates", l.gates ? list(l.gates, true) : undefined],
        ["Metrics", l.metrics ? list(l.metrics) : undefined],
        ["Case set", l.workload],
        ["First baseline", l.first_baseline],
        ["Blocked by", l.blocked_by?.length ? join(l.blocked_by.map(ticketLink)) : "nothing"],
        ["Planned views", views.length ? join(views, "; ") : undefined],
      ]),
    ),
    sets.length ? sets.map((set) => evaluationSet(m, set)) : win("Evaluations", empty("No evaluations recorded for this loop.")),
    win(
      "k-rust capabilities",
      caps.length
        ? table([{ label: "Capability", nowrap: true }, { label: "Status" }, { label: "Need" }], caps.map((c) => ({ id: c.id, href: route("krust", c.id), cells: [capLink(c.id), capabilityStatus(m, c.status), c.need] })))
        : empty("No k-rust capabilities recorded for this loop."),
    ),
    problems.length ? win("Problems", problemTable(problems), { aside: `${problems.length}` }) : null,
  );
}
