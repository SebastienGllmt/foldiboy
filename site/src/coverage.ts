// Joins the derived coverage universes with the authored classification.

import { resolve } from "node:path";
import type { Problem } from "./model";
import * as sources from "./sources";

export interface Cell {
  value: string;
  note?: string;
  authored: boolean;
}

export interface Dimension {
  id: string;
  title: string;
  derived: boolean;
  cells: Record<string, Cell>;
}

export interface CoverageDataset {
  id: string;
  title: string;
  universe: string;
  provenance: sources.Provenance[];
  notes: string[];
  columns: { id: string; title: string }[];
  rows: (Record<string, string | number | undefined> & { key: string })[];
  dimensions: Dimension[];
}

interface AuthoredDimension {
  dataset: string;
  id: string;
  title: string;
  default: string;
  default_note?: string;
}

interface AuthoredEntry {
  dataset: string;
  key: string;
  dimension: string;
  value: string;
  note?: string;
}

const FILE = "classification.toml";

const hex = (n: number) => n.toString(16).toUpperCase().padStart(2, "0");

// How each derived dataset is keyed and shown, and which dimensions the site computes from it.
type Universe = Omit<CoverageDataset, "notes" | "dimensions"> & { derived: Dimension[] };

async function universes(problems: Problem[]): Promise<Universe[]> {
  const out: Universe[] = [];

  const opcodes = await sources.loadOpcodes(problems);
  if (opcodes) {
    const rows = opcodes.rows.map((o) => ({ key: `${o.page === "cb" ? "CB " : ""}${hex(o.code)}`, page: o.page, code: o.code, mnemonic: o.mnemonic, group: o.group, vectors: o.vectorsTotal === undefined ? undefined : `${o.vectorsPassed}/${o.vectorsTotal}` }));
    const cells: Record<string, Cell> = {};
    for (const [i, o] of opcodes.rows.entries()) {
      const value = o.vectorsTotal === undefined ? "no-vectors" : o.vectorsPassed === o.vectorsTotal ? "pass" : "fail";
      cells[rows[i]!.key] = { value, note: o.failure, authored: false };
    }
    out.push({
      id: "opcodes",
      title: "Opcode slots",
      universe: "All 256 base and 256 CB-prefixed encodings (gbops), with SingleStepTests results for the reference",
      provenance: opcodes.provenance,
      columns: [{ id: "key", title: "Slot" }, { id: "mnemonic", title: "Mnemonic" }, { id: "group", title: "Group" }, { id: "vectors", title: "Vectors" }],
      rows,
      derived: [{ id: "vectors", title: "SingleStepTests", derived: true, cells }],
    });
  }

  const registers = await sources.loadHardwareRegisters(problems);
  if (registers) {
    out.push({
      id: "registers",
      title: "Hardware registers",
      universe: "Every register in the Pan Docs hardware register list, including CGB-only registers",
      provenance: registers.provenance,
      columns: [{ id: "key", title: "Address" }, { id: "name", title: "Name" }, { id: "description", title: "Description" }, { id: "access", title: "Access" }, { id: "models", title: "Models" }],
      rows: registers.rows.map((r) => ({ key: r.address, ...r })),
      derived: [],
    });
  }

  const memory = await sources.loadMemoryMap(problems);
  if (memory) {
    out.push({
      id: "memory",
      title: "Memory map",
      universe: "Every region of the Pan Docs memory map",
      provenance: memory.provenance,
      columns: [{ id: "key", title: "Start" }, { id: "end", title: "End" }, { id: "description", title: "Region" }, { id: "notes", title: "Notes" }],
      rows: memory.rows.map((r) => ({ key: r.start, ...r })),
      derived: [],
    });
  }

  const mooneye = await sources.loadMooneye(problems);
  if (mooneye) {
    out.push({
      id: "mooneye",
      title: "Mooneye acceptance tests",
      universe: "Every acceptance test source in the pinned mooneye test suite",
      provenance: mooneye.provenance,
      columns: [{ id: "key", title: "Test" }],
      rows: mooneye.rows.map((r) => ({ key: r.test })),
      derived: [],
    });
  }

  const state = await sources.loadStateCells(problems);
  if (state) {
    out.push({
      id: "state",
      title: "Reference state cells",
      universe: "Every configuration cell of the emitted reference definition",
      provenance: state.provenance,
      columns: [{ id: "key", title: "Cell" }, { id: "initial", title: "Initial value" }],
      rows: state.rows.map((r) => ({ key: r.cell, initial: r.initial })),
      derived: [],
    });
  }

  const claims = await sources.loadClaims(problems);
  if (claims) {
    const cells: Record<string, Cell> = {};
    for (const c of claims.rows) cells[c.claim] = { value: c.verdict === "proven" ? "proven" : c.verdict === "failed" ? "failed" : "indeterminate", authored: false };
    out.push({
      id: "claims",
      title: "Native claims",
      universe: "Every claim in the newest machine-readable native proof run",
      provenance: claims.provenance,
      columns: [{ id: "key", title: "Claim" }, { id: "module", title: "Spec module" }, { id: "modality", title: "Modality" }, { id: "source", title: "Source" }],
      rows: claims.rows.map((c) => ({ key: c.claim, module: c.module, modality: c.modality, source: c.source })),
      derived: [{ id: "verdict", title: "Verdict", derived: true, cells }],
    });
  }
  return out;
}

export async function loadCoverage(dir: string, statusIds: Set<string>, problems: Problem[]): Promise<CoverageDataset[]> {
  let doc: { dimension?: AuthoredDimension[]; entry?: AuthoredEntry[]; dataset_note?: { dataset: string; note: string }[] } = {};
  try {
    doc = Bun.TOML.parse(await Bun.file(resolve(dir, FILE)).text()) as typeof doc;
  } catch (error) {
    problems.push({ severity: "error", file: FILE, message: `cannot load: ${(error as Error).message}` });
  }

  const datasets: CoverageDataset[] = [];
  for (const u of await universes(problems)) {
    const keys = new Set(u.rows.map((r) => r.key));
    const dimensions: Dimension[] = [...u.derived];
    for (const d of (doc.dimension ?? []).filter((d) => d.dataset === u.id)) {
      if (!statusIds.has(d.default)) problems.push({ severity: "error", file: FILE, id: `${d.dataset}/${d.id}`, message: `unknown coverage status '${d.default}'` });
      const cells: Record<string, Cell> = {};
      for (const key of keys) cells[key] = { value: d.default, note: d.default_note, authored: false };
      dimensions.push({ id: d.id, title: d.title, derived: false, cells });
    }
    for (const e of (doc.entry ?? []).filter((e) => e.dataset === u.id)) {
      const dim = dimensions.find((d) => d.id === e.dimension && !d.derived);
      const where = `${e.dataset}/${e.dimension}/${e.key}`;
      if (!dim) problems.push({ severity: "error", file: FILE, id: where, message: `unknown dimension '${e.dimension}'` });
      else if (!keys.has(e.key)) problems.push({ severity: "error", file: FILE, id: where, message: `key is not in the ${u.id} universe` });
      else if (!statusIds.has(e.value)) problems.push({ severity: "error", file: FILE, id: where, message: `unknown coverage status '${e.value}'` });
      else dim.cells[e.key] = { value: e.value, note: e.note, authored: true };
    }
    for (const dim of dimensions) {
      const unclassified = Object.values(dim.cells).filter((c) => c.value === "unclassified").length;
      if (unclassified) problems.push({ severity: "info", file: FILE, id: `${u.id}/${dim.id}`, message: `${unclassified} of ${keys.size} rows unclassified` });
    }
    const notes = (doc.dataset_note ?? []).filter((n) => n.dataset === u.id).map((n) => n.note);
    datasets.push({ id: u.id, title: u.title, universe: u.universe, provenance: u.provenance, notes, columns: u.columns, rows: u.rows, dimensions });
  }
  for (const d of [...(doc.dimension ?? []), ...(doc.entry ?? []), ...(doc.dataset_note ?? [])]) {
    if (!datasets.some((x) => x.id === d.dataset) && !problems.some((p) => p.file === `source:${d.dataset}`)) {
      problems.push({ severity: "error", file: FILE, id: d.dataset, message: "unknown dataset" });
    }
  }
  return datasets;
}
