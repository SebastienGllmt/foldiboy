# Oracles

Harnesses that run external test suites against a semantics: one directory per suite, and in it one harness per semantics it runs on, named after that semantics (`sm83` is `krs/sm83`, the CPU stage).
The suites themselves are checkouts under `FOLDIBOY_TEST_ROOT` (by default `roms/tests` beside the repository), never copied here; what a harness builds or records stays beside it, untracked.

| Suite | Harness | Run with |
| --- | --- | --- |
| [SingleStepTests sm83](https://github.com/SingleStepTests/sm83) | `singlestep/sm83.ts`: converts each vector into a case document | `bun run sst` |
| [mooneye-test-suite](https://github.com/Gekkio/mooneye-test-suite) | `mooneye/sm83.toml`: the acceptance ROMs that need only the CPU, RAM, `IE` and `IF` | `bun run mooneye` |

They are independent evidence about the specification, and their tests are not corpus cases (`../../corpus/README.md`, "The corpus and external test suites").
`krs/sm83/README.md`, "Upstream conformance suites", states what each harness maps and what it does not.
