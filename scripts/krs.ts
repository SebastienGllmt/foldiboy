// Runs the pinned krs (krs/K_RUST_PIN) on the SM83 definition:
//
//   bun scripts/krs.ts VERB [ARGS...] [--save FILE]
//
// With --save, krs's output goes to FILE and only its summary (the last line) is printed.
// KRS_BIN overrides the krs binary.

import { mkdir, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";

export const root = resolve(import.meta.dir, "..");
const bin = process.env.KRS_BIN ?? "/artifacts/worktrees/k-rust-foldiboy-pin/target/debug/krs";

// krs reads the views package's Cargo workspace: a case's observer (krs/sm83-views) must be one
// of its packages, and the SM83 package is one too, as its dependency.
const views = resolve(root, "krs/sm83-views");
const definition = [
  "--manifest-path", `${views}/Cargo.toml`,
  "--package", "foldiboy-sm83", "--definition", "main",
  "--dependency", "foldiboy-sm83-core:core", "--dependency", "foldiboy-sm83-events:events",
];

// Runs `krs VERB ARGS...` from the repository root and returns its exit status. Its output is
// printed, or with `save` written to that file (relative to the root).
export async function krs(verb: string, args: string[], save?: string): Promise<number> {
  const corpus = verb === "kink" ? ["--corpus", resolve(root, "corpus")] : [];
  if (save) await mkdir(dirname(resolve(root, save)), { recursive: true });
  const child = Bun.spawn([bin, verb, ...args, ...definition, ...corpus], {
    cwd: root,
    env: { ...process.env, CARGO_TARGET_DIR: `${views}/target` },
    stdout: save ? Bun.file(resolve(root, save)) : "inherit",
    stderr: "inherit",
  });
  return await child.exited;
}

// Prints the last line of a file krs saved: its summary.
export async function printSummary(file: string) {
  const path = resolve(root, file);
  const { size } = await stat(path);
  const tail = await Bun.file(path).slice(Math.max(0, size - 65536), size).text();
  console.log(`${tail.trim().split("\n").at(-1)}\n(every record in ${file})`);
}

if (import.meta.main) {
  const [verb, ...args] = Bun.argv.slice(2);
  const at = args.indexOf("--save");
  const save = at >= 0 ? args.splice(at, 2)[1] : undefined;
  const status = await krs(verb!, args, save);
  if (save) await printSummary(save);
  process.exit(status);
}
