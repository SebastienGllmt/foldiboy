// Domain helpers shared by several views: links to records, and statuses for the model's own vocabularies.

import type { Model } from "../../src/model";
import { h, join } from "../ui/dom";
import { dialogue, link, route } from "../ui/components";
import { statusOf } from "../ui/status";

export const loopLink = (id: string) => link(route("loops", id), id);
export const ticketLink = (id: string) => link(route("tickets", id), id);
export const capLink = (id: string) => link(route("krust", id), id);
export const coverageLink = (id: string, text: string) => link(route("coverage", id), text, "");

export const capabilityStatus = (m: Model, id: string, label?: string) => statusOf(m.vocabulary.capability_status, id, label);
export const ticketStatus = (m: Model, id = "proposed", label?: string) => statusOf(m.vocabulary.ticket_status, id, label);
export const coverageStatus = (m: Model, id: string, label?: string) => statusOf(m.vocabulary.coverage_status, id, label);
export const decision = (m: Model, id: string) => statusOf(m.vocabulary.decision, id);

/** The dialogue shown on pages whose source files are not adopted yet. */
export function fileStatusNotice(m: Model, files?: string[]) {
  const pending = Object.entries(m.fileStatus).filter(([f, st]) => st !== "adopted" && (!files || files.includes(f)));
  if (!pending.length) return null;
  return dialogue(["Not adopted yet: ", join(pending.map(([f, st]) => h("span", { class: "nowrap" }, h("code", {}, f), ` (${st})`))), ". Treat these records as a proposal."], "warning");
}
