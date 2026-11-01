// The route table. The menu, the D-pad page cycle and the router are all derived from it.
// Adding a page = one view module + one entry here. See DESIGN.md, "Adding a page".

import type { Model } from "../../src/model";
import { coverageDetail, coverageIndex } from "./coverage";
import { guideView } from "./guide";
import { krustView } from "./krust";
import { loopDetail, loopsView } from "./loops";
import { overview } from "./overview";
import { problemsView } from "./problems";
import { ticketsView } from "./tickets";
import { vocabularyView } from "./vocabulary";

export interface Route {
  id: string;
  /** Menu label. */
  label: string;
  /** In the menu and the D-pad cycle (true), or reachable only by link (false). */
  menu: boolean;
  /** A short count shown beside the menu label, or nothing. */
  count?: (m: Model) => string | number | undefined;
  /** The list page. */
  list: (m: Model) => HTMLElement;
  /** The page for #/<id>/<record>. Without it, the record id selects the matching table row on the list page. */
  detail?: (m: Model, record: string) => HTMLElement;
}

export const ROUTES: Route[] = [
  { id: "overview", label: "Overview", menu: true, list: overview },
  { id: "loops", label: "Loops", menu: true, count: (m) => m.loops.length, list: loopsView, detail: loopDetail },
  { id: "tickets", label: "Tickets", menu: true, count: (m) => m.tickets.length, list: ticketsView },
  { id: "coverage", label: "Coverage", menu: true, count: (m) => m.coverage.length, list: coverageIndex, detail: coverageDetail },
  { id: "krust", label: "k-rust", menu: true, count: (m) => m.capabilities.length, list: krustView },
  { id: "vocabulary", label: "Vocabulary", menu: true, list: vocabularyView },
  { id: "problems", label: "Problems", menu: true, count: (m) => m.problems.length || undefined, list: problemsView },
  { id: "guide", label: "Style guide", menu: false, list: guideView },
];
