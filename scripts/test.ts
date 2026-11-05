// Runs the emulator's tests (the lib/ Cargo workspace) against the upstream test ROMs:
//
//   bun scripts/test.ts [CARGO TEST ARGS...]
//
// Builds SameSuite first (scripts/roms.ts); gb-test-roms ships its ROMs. Both are checkouts
// under FOLDIBOY_TEST_ROOT (default: roms/tests beside this repository); machine-docs/testing.md
// has their upstream URLs.

import { exists } from "node:fs/promises";
import { resolve } from "node:path";
import { $ } from "bun";
import { root } from "./krs";
import { testRoot } from "./roms";

for (const suite of ["gb-test-roms", "SameSuite"]) {
  if (!(await exists(resolve(testRoot, suite, ".git")))) {
    console.error(`Missing checkout: ${testRoot}/${suite}\nSee machine-docs/testing.md for upstream URLs.`);
    process.exit(1);
  }
}
await $`bun ${root}/scripts/roms.ts samesuite`;
const args = Bun.argv.slice(2);
const cargo = Bun.spawn(["cargo", "test", "--locked", "--workspace", "--no-fail-fast", ...args], {
  cwd: resolve(root, "lib"),
  env: { ...process.env, FOLDIBOY_TEST_ROOT: testRoot },
  stdio: ["inherit", "inherit", "inherit"],
});
process.exit(await cargo.exited);
