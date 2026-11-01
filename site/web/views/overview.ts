// Overview: where the project stands, then the tracking README.

import type { Model } from "../../src/model";
import { countBy, join } from "../ui/dom";
import { cols, counters, gauge, page, prose, route, win } from "../ui/components";
import { miniGrid, slotLegend } from "../ui/matrix";
import { fileStatusNotice } from "./common";
import { opcodeGrids, opcodeLegend } from "./coverage";

// README links to tracking files become site routes.
const FILE_ROUTES: Record<string, string> = {
  "loops.toml": route("loops"),
  "tickets.toml": route("tickets"),
  "krust-capabilities.toml": route("krust"),
  "vocabulary.toml": route("vocabulary"),
};

export function overview(m: Model) {
  const gates = m.evaluations.flatMap((set) => set.evaluation.flatMap((e) => e.gates));
  const passedGates = gates.filter((g) => g.verdict === "pass").length;
  const built = m.capabilities.filter((c) => c.status === "built").length;
  const specified = m.loops.filter((l) => l.maturity === "specified").length;
  const errors = m.problems.filter((p) => p.severity === "error").length;
  const opcodes = m.coverage.find((d) => d.id === "opcodes");
  const grids = opcodes ? opcodeGrids(m, opcodes) : [];
  const passing = grids.flatMap((g) => g.slots).filter((x) => x.tone === "good" && !x.marked).length;
  return page(
    { title: "Overview", sub: "foldiboy autoresearch" },
    fileStatusNotice(m),
    cols(
      win(
        "Counters",
        counters([
          { n: m.loops.length, label: "Loops", detail: join(countBy(m.loops, (l) => l.maturity).map(([k, n]) => `${n} ${k}`)), href: route("loops") },
          { n: m.gaps.length, label: "Gaps", detail: "no loop yet", href: route("loops") },
          { n: m.tickets.length, label: "Tickets", detail: `${m.tickets.filter((t) => t.status === "in-progress").length} in progress`, href: route("tickets") },
          { n: m.capabilities.length, label: "k-rust", detail: "capabilities", href: route("krust") },
          { n: m.coverage.length, label: "Universes", detail: "coverage", href: route("coverage") },
          { n: m.evaluations.reduce((n, s) => n + s.evaluation.length, 0), label: "Evaluations", detail: m.evaluations.length === 1 ? "1 epoch" : `${m.evaluations.length} epochs` },
          { n: m.problems.length, label: "Problems", detail: errors ? `${errors} errors` : "no errors", href: route("problems") },
        ]),
      ),
      win("Progress", [
        opcodes ? gauge("Opcodes pass", passing, 512, "good") : null,
        gauge("Gates pass", passedGates, gates.length, "good"),
        gauge("k-rust built", built, m.capabilities.length, "good"),
        gauge("Loops specified", specified, m.loops.length, "good"),
      ]),
    ),
    opcodes
      ? win("Opcode map", cols(grids.map((g, i) => miniGrid(g, i ? "CB page" : "Base page")), slotLegend(opcodeLegend(m, opcodes), true)), { aside: "512 slots" })
      : null,
    m.readmeHtml
      ? win(
          "README",
          prose(m.readmeHtml, (a) => {
            const to = FILE_ROUTES[a.getAttribute("href") ?? ""];
            if (to) a.setAttribute("href", to);
          }),
          { aside: "README.md" },
        )
      : null,
  );
}
