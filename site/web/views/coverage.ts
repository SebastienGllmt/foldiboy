// Coverage: the universes, and one page per universe. The opcode universe also gets the slot matrix.

import type { CoverageDataset, Dimension } from "../../src/coverage";
import type { Model } from "../../src/model";
import { countBy, h } from "../ui/dom";
import { empty, fields, link, list, page, route, small, split, table, win } from "../ui/components";
import { slotMatrix, type Grid, type LegendItem, type Slot } from "../ui/matrix";
import { termOf } from "../ui/status";
import { coverageLink, coverageStatus } from "./common";

const dimensionParts = (m: Model, dim: Dimension) => countBy(Object.values(dim.cells), (c) => c.value).map(([value, n]) => ({ label: value, n, tone: termOf(m.vocabulary.coverage_status, value).tone }));

const provenance = (d: CoverageDataset) =>
  list(d.provenance.map((p) => [h("code", {}, p.path), p.revision ? ` @ ${p.revision}` : null, p.modified ? small(` · modified ${p.modified.slice(0, 10)}`) : null]));

export function coverageIndex(m: Model) {
  return page(
    { title: "Coverage", sub: `${m.coverage.length} universes` },
    win(
      "Universes",
      [
        h("p", { class: "muted" }, "Each universe is derived from a project or reference source; authored classifications live in classification.toml. Unclassified rows are gaps."),
        table(
          [{ label: "Universe" }, { label: "Rows", num: true }, { label: "By dimension" }, { label: "Source" }],
          m.coverage.map((d) => ({
            id: d.id,
            href: route("coverage", d.id),
            cells: [[coverageLink(d.id, d.title), small(d.universe)], d.rows.length, d.dimensions.map((dim) => split(dimensionParts(m, dim), dim.title)), provenance(d)],
          })),
        ),
      ],
    ),
  );
}

// ---------- the opcode universe as slot grids ----------

export function opcodeGrids(m: Model, d: CoverageDataset): Grid[] {
  const dim = d.dimensions.find((x) => x.id === "vectors") ?? d.dimensions[0]!;
  return (["base", "cb"] as const).map((pageId) => {
    const rows = d.rows.filter((r) => r.page === pageId).sort((a, b) => Number(a.code) - Number(b.code));
    return {
      id: pageId,
      corner: pageId === "cb" ? "CB" : "",
      slots: rows.map((r): Slot => {
        const cell = dim.cells[r.key];
        const value = cell?.value ?? "unclassified";
        const t = termOf(m.vocabulary.coverage_status, value);
        const prefix = pageId === "base" && r.key === "CB";
        const key = r.key; // CB-page keys already read "CB xx"
        return {
          key,
          label: `${key} ${r.mnemonic}: ${prefix ? "prefix" : value}`,
          tone: t.tone,
          marked: prefix,
          glyph: prefix ? "CB" : t.tone === "critical" ? "✕" : undefined,
          detail: fields([
            ["Mnemonic", String(r.mnemonic ?? "")],
            ["Group", r.group === undefined ? undefined : String(r.group)],
            ["Vectors", r.vectors === undefined ? "none" : String(r.vectors)],
            ["Status", prefix ? "prefix: opens the CB page" : coverageStatus(m, value)],
            ["Note", cell?.note ? small(cell.note) : undefined],
          ]),
        };
      }),
    };
  });
}

export function opcodeLegend(m: Model, d: CoverageDataset): LegendItem[] {
  const dim = d.dimensions.find((x) => x.id === "vectors") ?? d.dimensions[0]!;
  const isPrefix = (r: CoverageDataset["rows"][number]) => r.page === "base" && r.key === "CB";
  const values = d.rows.filter((r) => !isPrefix(r)).map((r) => dim.cells[r.key]?.value ?? "unclassified");
  const items: LegendItem[] = countBy(values, (v) => v).map(([value, n]) => {
    const t = termOf(m.vocabulary.coverage_status, value);
    return { tone: t.tone, glyph: t.tone === "critical" ? "\u2715" : undefined, label: value, n, meaning: t.meaning };
  });
  const prefixes = d.rows.filter(isPrefix).length;
  if (prefixes) items.push({ tone: "neutral", marked: true, glyph: "CB", label: "prefix", n: prefixes, meaning: "opens the CB page" });
  return items;
}

export function coverageDetail(m: Model, id: string) {
  const d = m.coverage.find((x) => x.id === id);
  const crumb = { href: route("coverage"), label: "Coverage" };
  if (!d) return page({ title: "Unknown universe", crumb }, empty(`No coverage universe is called ${id}.`));
  const dimCell = (dim: Dimension, key: string) => {
    const c = dim.cells[key];
    if (!c) return "";
    return [coverageStatus(m, c.value), c.note && (c.authored || dim.derived) ? small(c.note) : null];
  };
  const isOpcodes = id === "opcodes";
  const grids = isOpcodes ? opcodeGrids(m, d) : [];
  const detailRows = isOpcodes ? d.rows.filter((r) => d.dimensions[0]!.cells[r.key]?.value !== "pass") : d.rows;
  const firstFail = grids[0]?.slots.findIndex((x) => x.tone === "critical") ?? -1;
  return page(
    { title: d.title, sub: d.universe, crumb },
    d.notes.map((n) => win("Note", n)),
    isOpcodes ? win("Slot map", slotMatrix(grids, opcodeLegend(m, d), [0, Math.max(0, firstFail)]), { aside: `${grids.flatMap((g) => g.slots).length} slots` }) : null,
    win(
      "Summary",
      table(
        [{ label: "Dimension" }, { label: "Default when unlisted" }, { label: "Rows by status" }],
        d.dimensions.map((dim) => ({ cells: [dim.title, dim.derived ? small("derived") : (Object.values(dim.cells).find((c) => !c.authored)?.note ?? ""), split(dimensionParts(m, dim))] })),
      ),
    ),
    win(
      isOpcodes ? "Slots that do not pass" : "Rows",
      detailRows.length
        ? table(
            [...d.columns.map((c) => ({ label: c.title, nowrap: c.id === "key" })), ...d.dimensions.map((dim) => ({ label: dim.title }))],
            detailRows.map((r) => ({ id: r.key, cells: [...d.columns.map((c) => String(r[c.id] ?? "")), ...d.dimensions.map((dim) => dimCell(dim, r.key))] })),
          )
        : empty("Every row passes."),
      { aside: `${detailRows.length} rows` },
    ),
    win("Source", provenance(d)),
    h("p", {}, link(route("coverage"), "◀ All universes", "")),
  );
}
