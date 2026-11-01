// The route table. The menu, the D-pad page cycle, the router and the info screen's page descriptions are all derived from it.
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
  /** In the menu and the D-pad cycle (true), or reachable only by URL or link (false), like the style guide at #/guide. */
  menu: boolean;
  /** One sentence on what the page shows, read out on the info screen when nothing is selected. */
  about: string;
  /** The same for the detail page (#/<id>/<record>). */
  detailAbout?: string;
  /** A short count shown beside the menu label, or nothing. */
  count?: (m: Model) => string | number | undefined;
  /** The list page. */
  list: (m: Model) => HTMLElement;
  /** The page for #/<id>/<record>. Without it, the record id selects the matching table row on the list page. */
  detail?: (m: Model, record: string) => HTMLElement;
}

export const ROUTES: Route[] = [
  { id: "overview", label: "Overview", menu: true, about: "Counters for every tracking file, the opcode map in miniature, and the tracking README.", list: overview },
  {
    id: "loops", label: "Loops", menu: true, count: (m) => m.loops.length, list: loopsView, detail: loopDetail,
    about: "Every research loop by area, then the gaps no loop covers yet.",
    detailAbout: "One loop: its definition, its evaluations and the k-rust capabilities it needs.",
  },
  { id: "tickets", label: "Tickets", menu: true, count: (m) => m.tickets.length, list: ticketsView, about: "Work items with their status, what they depend on and what they unblock." },
  {
    id: "coverage", label: "Coverage", menu: true, count: (m) => m.coverage.length, list: coverageIndex, detail: coverageDetail,
    about: "Each coverage universe and how its rows split by status.",
    detailAbout: "Every row of one universe. On a slot matrix the D-pad moves the cursor and this screen reads out the slot.",
  },
  { id: "krust", label: "k-rust", menu: true, count: (m) => m.capabilities.length, list: krustView, about: "What the k-rust toolchain can and cannot do yet, and which loops need it." },
  { id: "vocabulary", label: "Vocabulary", menu: true, list: vocabularyView, about: "The terms the tracking files use for shapes, evidence, maturity and status." },
  { id: "problems", label: "Problems", menu: true, count: (m) => m.problems.length || undefined, list: problemsView, about: "Checks the tracking files fail today." },
  { id: "guide", label: "Style guide", menu: false, list: guideView, about: "Every token, sprite and component, drawn by the code the pages use. Rules are in site/DESIGN.md." },
];
