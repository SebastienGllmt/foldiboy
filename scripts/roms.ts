// Assembles the test ROMs:
//
//   bun scripts/roms.ts [corpus] [mooneye] [samesuite]     (all three by default)
//
//   corpus     corpus/cases/<id>/case.asm  ->  corpus/cases/<id>/case.gb, with RGBDS
//   mooneye    the mooneye tests krs/oracles/mooneye/sm83.toml runs  ->  krs/oracles/mooneye/<test>.gb, with WLA-DX
//   samesuite  every SameSuite test, in its own checkout, as its Makefile does
//
// The upstream suites are checkouts under FOLDIBOY_TEST_ROOT (default: roms/tests beside this
// repository). The assemblers come from nixpkgs. Progress goes to stderr.

import { existsSync, realpathSync } from "node:fs";
import { readdir, rm } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { $ } from "bun";
import { root } from "./krs";

// The workspace may reach this repository through a symlink, and roms/ is beside the symlink:
// prefer the logical path the command was started from.
function logicalRoot(): string {
  for (let dir = process.env.PWD; dir && dir !== dirname(dir); dir = dirname(dir)) {
    if (realpathSync(dir) === root) return dir;
  }
  return root;
}
export const testRoot = process.env.FOLDIBOY_TEST_ROOT ?? resolve(logicalRoot(), "../roms/tests");

const MOONEYE_TESTS = ["ei_sequence", "ei_timing", "rapid_di_ei", "reti_intr_timing", "interrupts/ie_push", "if_ie_registers"];

// An upstream suite's checkout.
export function checkout(name: string): string {
  const dir = resolve(testRoot, name);
  if (!existsSync(dir)) throw new Error(`missing checkout ${dir}; set FOLDIBOY_TEST_ROOT (machine-docs/testing.md)`);
  return dir;
}

// The bin/ directory of a nixpkgs package (an installable such as `nixpkgs#rgbds`).
async function tools(installable: string): Promise<string> {
  return (await $`nix build ${installable} --no-link --print-out-paths`.text()).trim() + "/bin";
}

// Every byte a case does not place is $DD, the lock opcode, so a jump outside the case's code
// ends the run at once.
async function corpus() {
  const bin = await tools("nixpkgs#rgbds");
  const ids = await readdir(resolve(root, "corpus/cases"));
  for (const id of ids) {
    const dir = resolve(root, "corpus/cases", id);
    await $`${bin}/rgbasm -Wall -I ${root}/corpus/include -o ${dir}/case.o ${dir}/case.asm`.quiet();
    await $`${bin}/rgblink -p 0xDD -o ${dir}/case.gb -n ${dir}/case.sym -m ${dir}/case.map ${dir}/case.o`.quiet();
    await $`${bin}/rgbfix -v -p 0xDD ${dir}/case.gb`.quiet();
    await rm(`${dir}/case.o`);
  }
  console.error(`corpus: ${ids.length} ROMs`);
}

async function mooneye() {
  const bin = await tools("nixpkgs#wla-dx");
  const suite = checkout("mooneye-test-suite");
  const out = resolve(root, "krs/oracles/mooneye");
  for (const test of MOONEYE_TESTS) {
    const name = basename(test);
    await $`${bin}/wla-gb -I ${suite}/common -o ${out}/${name}.o ${suite}/acceptance/${test}.s`.quiet();
    await Bun.write(`${out}/${name}.link`, `[objects]\n${out}/${name}.o\n`);
    await $`${bin}/wlalink -d -S ${out}/${name}.link ${out}/${name}.gb`.quiet();
    await rm(`${out}/${name}.o`);
    await rm(`${out}/${name}.link`);
  }
  console.error(`mooneye: ${MOONEYE_TESTS.length} ROMs`);
}

// SameSuite is built with the RGBDS release it has always been built with here (a pinned nixpkgs).
async function samesuite() {
  const bin = await tools("github:NixOS/nixpkgs/c8f90650c15282fa8656a041bfbbd2403997a9a7#rgbds");
  const suite = checkout("SameSuite");
  const sources = (await readdir(suite, { recursive: true })).filter((file) => file.endsWith(".asm"));
  for (const source of sources) {
    const stem = resolve(suite, source.slice(0, -".asm".length));
    await $`${bin}/rgbasm -I include/ -o ${stem}.o ${stem}.asm`.cwd(suite).quiet();
    await $`${bin}/rgblink -o ${stem}.gb -n ${stem}.sym ${stem}.o`.cwd(suite).quiet();
    await $`${bin}/rgbfix -jv ${stem}.gb`.cwd(suite).quiet();
    await rm(`${stem}.o`);
  }
  console.error(`samesuite: ${sources.length} ROMs`);
}

if (import.meta.main) {
  const sets = { corpus, mooneye, samesuite };
  const chosen = Bun.argv.slice(2);
  try {
    for (const name of chosen.length ? chosen : Object.keys(sets)) {
      const build = sets[name as keyof typeof sets];
      if (!build) throw new Error(`unknown ROM set ${name}; expected ${Object.keys(sets).join(", ")}`);
      await build();
    }
  } catch (error) {
    // A failed assembler command: show what it printed.
    if (error instanceof $.ShellError) console.error(`${error.stdout}${error.stderr}`);
    throw error;
  }
}
