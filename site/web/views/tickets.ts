// Tickets: work items by phase, with what each depends on and what it unblocks. #/tickets/<id> selects that ticket's row.

import type { Model } from "../../src/model";
import { countBy, h, join } from "../ui/dom";
import { chips, list, page, small, table, win } from "../ui/components";
import { fileStatusNotice, loopLink, ticketLink, ticketStatus } from "./common";

export function ticketsView(m: Model) {
  const phases = [...new Set(m.tickets.map((t) => t.phase))].sort((a, b) => a - b);
  return page(
    { title: "Tickets", sub: `${m.tickets.length} tickets` },
    fileStatusNotice(m, ["tickets.toml"]),
    win("By status", chips(countBy(m.tickets, (t) => t.status ?? "proposed").map(([st, n]) => ticketStatus(m, st, `${st} ${n}`)))),
    phases.map((phase) => {
      const tickets = m.tickets.filter((t) => t.phase === phase);
      return win(
        `Phase ${phase}`,
        table(
          [{ label: "Status", nowrap: true }, { label: "Ticket", nowrap: true }, { label: "Title and work" }, { label: "Depends on" }, { label: "Unblocks" }],
          tickets.map((t) => ({
            id: t.id,
            cells: [
              ticketStatus(m, t.status),
              t.id,
              [t.title, small(t.work), t.note ? small(t.note) : null, t.open?.length ? [small("Open questions:"), list(t.open.map((q) => small(q)))] : null],
              join((t.depends_on ?? []).map(ticketLink)),
              join([...(m.derived.ticketDependents[t.id] ?? []).map(ticketLink), ...(m.derived.ticketBlocksLoops[t.id] ?? []).map((l) => h("span", { class: "nowrap" }, "loop ", loopLink(l)))]),
            ],
          })),
        ),
        { aside: `${tickets.length} tickets` },
      );
    }),
  );
}
