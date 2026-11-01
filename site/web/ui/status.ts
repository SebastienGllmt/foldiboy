// Status: how any recorded status becomes a tone, and how a tone is shown.
// Tones come from vocabulary.toml (each term has `tone`); this file holds the few mappings that are not in the vocabulary.
// Rule: a status is always drawn with status() or statusOf(): sprite + label. Never colour alone. See DESIGN.md, "Status".

import type { Severity, Term } from "../../src/model";
import { h } from "./dom";
import { isTone, toneSprite, type Tone } from "./sprites";

/** A tone, a label and an optional explanation shown on hover. */
export function status(tone: Tone, label: string, title?: string): HTMLElement {
  return h("span", { class: `status tone-${tone}`, title }, toneSprite(tone), label);
}

/** The term with this id in a vocabulary list; unknown ids become neutral so they still render and stand out as unclassified. */
export function termOf(terms: Term[] | undefined, id: string): { id: string; tone: Tone; meaning?: string } {
  const term = terms?.find((t) => t.id === id);
  return { id, tone: isTone(term?.tone) ? term.tone : "neutral", meaning: term?.meaning };
}

/** A vocabulary status: its tone's sprite, its id as the label, its meaning on hover. */
export function statusOf(terms: Term[] | undefined, id: string, label: string = id): HTMLElement {
  const t = termOf(terms, id);
  return status(t.tone, label, t.meaning);
}

// Statuses that live in code rather than in vocabulary.toml.
export const SEVERITY_TONE: Record<Severity, Tone> = { error: "critical", warning: "warning", info: "neutral" };
export const GATE_TONE: Record<string, Tone> = { pass: "good", fail: "critical", open: "warning" };
export const severity = (s: Severity) => status(SEVERITY_TONE[s], s);
export const gateVerdict = (v: string) => status(GATE_TONE[v] ?? "neutral", v);
