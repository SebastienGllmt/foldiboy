// k-rust: what the toolchain can do for each loop. #/krust/<id> selects that capability's row.

import type { Model } from "../../src/model";
import { h, join } from "../ui/dom";
import { chips, empty, page, small, table, win } from "../ui/components";
import { capabilityStatus, fileStatusNotice, loopLink } from "./common";

export function krustView(m: Model) {
  const without = m.loops.filter((l) => !m.derived.loopCapabilities[l.id]);
  return page(
    { title: "k-rust capabilities", sub: `${m.capabilities.length} recorded` },
    fileStatusNotice(m, ["krust-capabilities.toml"]),
    win("By status", chips(m.vocabulary.capability_status.map((t) => capabilityStatus(m, t.id, `${t.id} ${m.capabilities.filter((c) => c.status === t.id).length}`)))),
    win(
      "Capabilities",
      table(
        [{ label: "Status", nowrap: true }, { label: "Capability", nowrap: true }, { label: "Need" }, { label: "Loops" }, { label: "Source and notes" }],
        m.capabilities.map((c) => ({ id: c.id, cells: [capabilityStatus(m, c.status), c.id, c.need, join(c.loops.map(loopLink)), [c.source ? h("code", {}, c.source) : null, c.note ? small(c.note) : null]] })),
      ),
    ),
    win("Loops with no recorded capabilities", without.length ? h("p", {}, join(without.map((l) => loopLink(l.id)))) : empty("Every loop records its k-rust needs.")),
  );
}
