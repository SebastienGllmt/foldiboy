// Loads the tracking files, checks their cross-references and derives reverse links.
// The site renders this model; it never writes to the tracking directory.

import { resolve } from "node:path";
import { loadDocuments, type Document } from "./documents";
import { loadCoverage, type CoverageDataset } from "./coverage";
import { loadEvaluations, type EvaluationSet } from "./evaluations";

export type Severity = "error" | "warning" | "info";

export interface Problem {
  severity: Severity;
  file: string;
  id?: string;
  message: string;
}

type Row = Record<string, unknown>;

export interface Loop extends Row {
  id: string;
  maturity: string;
  shape: string[];
  area: string;
  objective: string;
  candidates?: string;
  gates?: string[];
  metrics?: string[];
  workload?: string;
  blocked_by?: string[];
  first_baseline?: string;
}

export interface Gap extends Row {
  id: string;
  layer: string;
  description: string;
}

export interface Ticket extends Row {
  id: string;
  phase: number;
  title: string;
  work: string;
  depends_on?: string[];
  status?: string;
  open?: string[];
  note?: string;
}

export interface Capability extends Row {
  id: string;
  loops: string[];
  need: string;
  status: string;
  source?: string;
  note?: string;
}

export interface Term extends Row {
  id: string;
  meaning?: string;
  progress?: string;
  examples?: string;
  views?: string[];
  tone?: string;
}

export interface Vocabulary {
  shape: Term[];
  evidence_level: Term[];
  maturity: Term[];
  capability_status: Term[];
  ticket_status: Term[];
  coverage_status: Term[];
  decision: Term[];
}

export interface Model {
  dir: string;
  loadedAt: string;
  fileStatus: Record<string, string>;
  readmeHtml: string;
  documents: Document[];
  loops: Loop[];
  gaps: Gap[];
  tickets: Ticket[];
  capabilities: Capability[];
  vocabulary: Vocabulary;
  coverage: CoverageDataset[];
  evaluations: EvaluationSet[];
  derived: {
    ticketDependents: Record<string, string[]>;
    ticketBlocksLoops: Record<string, string[]>;
    loopCapabilities: Record<string, string[]>;
  };
  problems: Problem[];
}

export const FILES = {
  readme: "README.md",
  loops: "loops.toml",
  tickets: "tickets.toml",
  capabilities: "krust-capabilities.toml",
  vocabulary: "vocabulary.toml",
} as const;

// Field names each record type accepts; an unknown field is reported, since it is usually a typo.
const FIELDS: Record<string, string[]> = {
  loop: ["id", "maturity", "shape", "area", "objective", "candidates", "gates", "metrics", "workload", "blocked_by", "first_baseline"],
  gap: ["id", "layer", "description"],
  ticket: ["id", "phase", "status", "title", "work", "depends_on", "open", "note"],
  capability: ["id", "loops", "need", "status", "source", "note"],
  shape: ["id", "progress", "examples", "views"],
  evidence_level: ["id", "meaning"],
  maturity: ["id", "meaning"],
  capability_status: ["id", "meaning", "tone"],
  ticket_status: ["id", "meaning", "tone"],
  coverage_status: ["id", "meaning", "tone"],
  decision: ["id", "meaning", "tone"],
};

const REQUIRED: Record<string, string[]> = {
  loop: ["id", "maturity", "shape", "area", "objective"],
  gap: ["id", "layer", "description"],
  ticket: ["id", "phase", "title", "work"],
  capability: ["id", "loops", "need", "status"],
};

const SPECIFIED_LOOP_FIELDS = ["candidates", "gates", "metrics", "workload"];

export function trackingDir(): string {
  return resolve(process.env.FOLDIBOY_TRACKING_DIR ?? resolve(import.meta.dir, "../../draft-personal/autoresearch"));
}

async function readToml(dir: string, file: string, problems: Problem[]): Promise<Row> {
  try {
    return Bun.TOML.parse(await Bun.file(resolve(dir, file)).text()) as Row;
  } catch (error) {
    problems.push({ severity: "error", file, message: `cannot load: ${(error as Error).message}` });
    return {};
  }
}

function table<T extends Row>(doc: Row, key: string, file: string, problems: Problem[]): T[] {
  const value = doc[key];
  if (value === undefined) return [];
  if (!Array.isArray(value)) {
    problems.push({ severity: "error", file, message: `[[${key}]] is not an array of tables` });
    return [];
  }
  const rows = value as T[];
  const seen = new Set<string>();
  for (const row of rows) {
    const id = typeof row.id === "string" ? row.id : undefined;
    for (const field of REQUIRED[key] ?? []) {
      if (row[field] === undefined) problems.push({ severity: "error", file, id, message: `${key} lacks required field '${field}'` });
    }
    for (const field of Object.keys(row)) {
      if (!(FIELDS[key] ?? []).includes(field)) problems.push({ severity: "warning", file, id, message: `${key} has unknown field '${field}'` });
    }
    if (id !== undefined) {
      if (seen.has(id)) problems.push({ severity: "error", file, id, message: `duplicate ${key} id` });
      seen.add(id);
    }
  }
  return rows;
}

function checkRefs(values: string[] | undefined, known: Set<string>, what: string, file: string, id: string, problems: Problem[]) {
  for (const value of values ?? []) {
    if (!known.has(value)) problems.push({ severity: "error", file, id, message: `unknown ${what} '${value}'` });
  }
}

function findCycles(tickets: Ticket[], problems: Problem[]) {
  const deps = new Map(tickets.map((t) => [t.id, t.depends_on ?? []]));
  const state = new Map<string, "active" | "done">();
  const visit = (id: string, path: string[]) => {
    if (state.get(id) === "done") return;
    if (state.get(id) === "active") {
      problems.push({ severity: "error", file: FILES.tickets, id, message: `dependency cycle: ${[...path.slice(path.indexOf(id)), id].join(" → ")}` });
      return;
    }
    state.set(id, "active");
    for (const dep of deps.get(id) ?? []) if (deps.has(dep)) visit(dep, [...path, id]);
    state.set(id, "done");
  };
  for (const t of tickets) visit(t.id, []);
}

function push(map: Record<string, string[]>, key: string, value: string) {
  (map[key] ??= []).push(value);
}

export async function loadModel(dir = trackingDir()): Promise<Model> {
  const problems: Problem[] = [];
  const [loopsDoc, ticketsDoc, capsDoc, vocabDoc] = await Promise.all([
    readToml(dir, FILES.loops, problems),
    readToml(dir, FILES.tickets, problems),
    readToml(dir, FILES.capabilities, problems),
    readToml(dir, FILES.vocabulary, problems),
  ]);

  let readmeHtml = "";
  try {
    readmeHtml = Bun.markdown.html(await Bun.file(resolve(dir, FILES.readme)).text());
  } catch (error) {
    problems.push({ severity: "error", file: FILES.readme, message: `cannot load: ${(error as Error).message}` });
  }

  const vocabulary: Vocabulary = {
    shape: table<Term>(vocabDoc, "shape", FILES.vocabulary, problems),
    evidence_level: table<Term>(vocabDoc, "evidence_level", FILES.vocabulary, problems),
    maturity: table<Term>(vocabDoc, "maturity", FILES.vocabulary, problems),
    capability_status: table<Term>(vocabDoc, "capability_status", FILES.vocabulary, problems),
    ticket_status: table<Term>(vocabDoc, "ticket_status", FILES.vocabulary, problems),
    coverage_status: table<Term>(vocabDoc, "coverage_status", FILES.vocabulary, problems),
    decision: table<Term>(vocabDoc, "decision", FILES.vocabulary, problems),
  };
  const loops = table<Loop>(loopsDoc, "loop", FILES.loops, problems);
  const gaps = table<Gap>(loopsDoc, "gap", FILES.loops, problems);
  const tickets = table<Ticket>(ticketsDoc, "ticket", FILES.tickets, problems);
  const capabilities = table<Capability>(capsDoc, "capability", FILES.capabilities, problems);

  const ids = (rows: Row[]) => new Set(rows.map((r) => r.id as string));
  const shapeIds = ids(vocabulary.shape);
  const maturityIds = ids(vocabulary.maturity);
  const statusIds = ids(vocabulary.capability_status);
  const ticketStatusIds = ids(vocabulary.ticket_status);
  const loopIds = ids(loops);
  const ticketIds = ids(tickets);

  const derived: Model["derived"] = { ticketDependents: {}, ticketBlocksLoops: {}, loopCapabilities: {} };

  for (const loop of loops) {
    checkRefs(loop.shape, shapeIds, "shape", FILES.loops, loop.id, problems);
    checkRefs([loop.maturity], maturityIds, "maturity", FILES.loops, loop.id, problems);
    checkRefs(loop.blocked_by, ticketIds, "ticket", FILES.loops, loop.id, problems);
    for (const t of loop.blocked_by ?? []) push(derived.ticketBlocksLoops, t, loop.id);
    if (loop.maturity === "specified") {
      for (const field of SPECIFIED_LOOP_FIELDS) {
        if (loop[field] === undefined) problems.push({ severity: "warning", file: FILES.loops, id: loop.id, message: `specified loop lacks '${field}'` });
      }
    }
  }
  for (const ticket of tickets) {
    checkRefs(ticket.depends_on, ticketIds, "ticket", FILES.tickets, ticket.id, problems);
    if (ticket.status !== undefined) checkRefs([ticket.status], ticketStatusIds, "ticket status", FILES.tickets, ticket.id, problems);
    for (const dep of ticket.depends_on ?? []) push(derived.ticketDependents, dep, ticket.id);
  }
  findCycles(tickets, problems);
  for (const cap of capabilities) {
    checkRefs(cap.loops, loopIds, "loop", FILES.capabilities, cap.id, problems);
    checkRefs([cap.status], statusIds, "capability status", FILES.capabilities, cap.id, problems);
    for (const l of cap.loops ?? []) push(derived.loopCapabilities, l, cap.id);
  }
  for (const loop of loops) {
    if (loop.maturity === "specified" && !derived.loopCapabilities[loop.id]) {
      problems.push({ severity: "info", file: FILES.capabilities, id: loop.id, message: "specified loop has no recorded k-rust capabilities" });
    }
  }

  const coverage = await loadCoverage(dir, ids(vocabulary.coverage_status), problems);
  const evaluations = await loadEvaluations(dir, { loops: loopIds, decisions: ids(vocabulary.decision), evidence: ids(vocabulary.evidence_level) }, problems);

  const fileStatus: Record<string, string> = {};
  for (const [file, doc] of [[FILES.loops, loopsDoc], [FILES.tickets, ticketsDoc], [FILES.capabilities, capsDoc], [FILES.vocabulary, vocabDoc]] as const) {
    if (typeof doc.status === "string") fileStatus[file] = doc.status;
  }

  const documents = await loadDocuments();
  return { documents, dir, loadedAt: new Date().toISOString(), fileStatus, readmeHtml, loops, gaps, tickets, capabilities, vocabulary, coverage, evaluations, derived, problems };
}
