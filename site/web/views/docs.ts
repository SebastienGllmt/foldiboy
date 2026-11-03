import type { Model } from "../../src/model";
import { dialogue, link, page, prose, route, table, win } from "../ui/components";

export function docsIndex(m: Model) {
  return page(
    { title: "Documentation", sub: "guides, architecture and proving design" },
    win("Start here", [
      link(route("docs", "docs/README.md"), "Getting started"),
      " · ", link(route("docs", "README.md"), "Project overview"),
      " · ", link(route("docs", "machine-docs/testing.md"), "Testing"),
    ]),
    ...["Project guides", "Managed documentation", "AI-assisted documentation"].map((group) => win(group,
      table([{ label: "Document" }, { label: "Source" }], m.documents.filter((d) => d.group === group).map((d) => ({
        id: d.path, href: route("docs", d.path), cells: [link(route("docs", d.path), d.title), d.path],
      }))),
    )),
  );
}

export function docsDetail(m: Model, record: string) {
  const [path = "", anchor] = record.split("#");
  const doc = m.documents.find((d) => d.path === path);
  const crumb = { href: route("docs"), label: "Documentation" };
  if (!doc) return page({ title: "Document not found", crumb }, dialogue(`No document is registered for ${path}.`));
  const body = prose(doc.html, (a) => {
    const href = a.getAttribute("href")!;
    if (/^(?:[a-z][a-z\d+.-]*:|\/\/)/i.test(href)) return;
    const target = new URL(href, `https://repository.local/${path}`);
    let destination = decodeURIComponent(target.pathname.slice(1)).replace(/\/$/, "");
    if (destination === "machine-docs") {
      a.href = route("docs");
      return;
    }
    if (destination === "docs") destination += "/README.md";
    if (m.documents.some((d) => d.path === destination)) {
      a.href = route("docs", destination + target.hash);
    } else {
      a.href = `https://github.com/SebastienGllmt/foldiboy/blob/HEAD/${destination}${target.hash}`;
    }
  });
  // GitHub-style heading ids keep the existing documents' section links usable.
  const ids = new Map<string, number>();
  body.querySelectorAll("h1,h2,h3,h4,h5,h6").forEach((heading) => {
    const slug = (heading.textContent ?? "").toLowerCase().replace(/[^\p{L}\p{N}\p{M}_\-\s]/gu, "").replace(/\s/g, "-");
    const n = ids.get(slug) ?? 0;
    ids.set(slug, n + 1);
    heading.id = n ? `${slug}-${n}` : slug;
  });
  if (anchor) requestAnimationFrame(() => {
    const id = decodeURIComponent(anchor);
    Array.from(body.querySelectorAll("[id]")).find((el) => el.id === id)?.scrollIntoView({ block: "start" });
  });
  return page({ title: doc.title, sub: doc.path, crumb }, win(doc.group, body));
}
