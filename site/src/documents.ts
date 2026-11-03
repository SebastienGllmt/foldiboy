// Repository Markdown is the source of truth; no generated documentation is stored.
import { resolve } from "node:path";

export const repositoryDir = resolve(import.meta.dir, "../..");
export interface Document {
  path: string;
  title: string;
  group: string;
  html: string;
}

export async function loadDocuments(): Promise<Document[]> {
  const paths = ["README.md", "docs/README.md", "host/desktop/README.md", "host/desktop/vendor/README.md", "krs/sm83/README.md", "krs/sm83/src/claims/README.md", "roms/README.md", "site/README.md", "site/DESIGN.md"];
  for (const directory of ["docs", "machine-docs"]) {
    for await (const path of new Bun.Glob("*.md").scan(resolve(repositoryDir, directory))) {
      if (!paths.includes(`${directory}/${path}`)) paths.push(`${directory}/${path}`);
    }
  }
  return Promise.all(paths.map(async (path) => {
    const text = await Bun.file(resolve(repositoryDir, path)).text();
    const group = path.startsWith("machine-docs/") ? "AI-assisted documentation" : path.startsWith("docs/") ? "Managed documentation" : "Project guides";
    return { path, title: /^# (.+)$/m.exec(text)?.[1] ?? path, group, html: Bun.markdown.html(text) };
  }));
}
