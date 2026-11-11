# Corpus

Programs every implementation of the Game Boy CPU runs, and the runner that compares them
(`draft-personal/corpus-contract`: decisions D-CASE-*, plan packages R7 and R8).

## A case

`cases/<id>/case.asm` is a cartridge in RGBDS syntax, and the case is the cartridge itself: `case.gb`, which `bun run roms` assembles beside it (ignored by git).
`krs run cases/<id>/case.gb`, `bun run kink` and `bun run compare` all start from that file.

`scheduled/<id>/` holds the cases that need the rest of the console: `case.asm`, assembled the same way, and `case.toml`, a case document that binds the cartridge and the interrupt requests the console makes (`requests`, the CPU stage's `REQUESTS` input; `krs/sm83/src/interrupt.rs`).
A request `{ At = { cycle = N, bits = B } }` sets `B` in `IF` once `N` machine cycles have run, which the bus log counts: one event per machine cycle.
Only a request can wake the CPU from `HALT` mode or arrive during an interrupt dispatch, since no program runs then. A case in `cases/` makes no request.
It starts at `$0100` in the state the DMG boot ROM leaves, jumps to `Start`, and ends by locking the CPU with an unused opcode (`LOCK`, or `DUMP_AND_LOCK`, which first pushes every register and stores `SP` at `$C000`).
Two kinds of case end otherwise, as the definition does: `stop` ends stuck on `STOP`, which the CPU stage does not model, and a case that enters `HALT` mode with nothing to wake it never wakes, so its run ends at its second halted boundary (`FLOW.toml` `[stop]`, and the same stop in `bun run compare`).
`include/case.inc` holds the entry, the header space and those macros.

A case sets no register or memory directly: it writes what it needs in a prologue.
RAM starts empty, and a read of a byte the case never wrote ends the run as stuck. The definition and the Rust adapter treat it the same way.

## The machine

Every case starts on a DMG after its boot ROM, in the CPU stage: the definition's initializer builds that state from the cartridge with the specification's own boot functions (`krs/sm83/src/boot.rs`).
The Rust CPU starts from the state the definition's initializer builds from the case, read from its configuration, so there is one statement of the boot state.

## The corpus and external test suites

kink makes the corpus robust for the specification.
Each case is there to close something kink computes from the definition, and kink reports when a case stops being needed: a rule to fire, a guard outcome to show, an encoding to execute, a mutation to kill.
Like any formal method, kink cannot tell whether the specification says what the hardware does.
That takes evidence from outside the specification.

The external suites are that evidence: SingleStepTests and the mooneye ROMs (`krs/oracles/`), and later SameSuite and other test ROMs.
Each runs against the definition through its own harness, and none of their tests is copied into the corpus:

- A corpus case states its purpose, and kink checks it. A copied test states none, so nothing could say what it protects or when it becomes redundant.
- A suite that helped build the corpus would no longer be independent evidence about it.

When an external suite finds a defect the corpus misses, the definition is fixed. If the corpus also failed to notice the defect, its process is fixed too: a case for an obligation kink already states, or the kind of obligation kink is missing, raised with k-rust.

## Comparing implementations

FLOW.toml declares the other implementations of the CPU (`[[implementation]]`), and `krs kink compare` runs each on every corpus program and compares its end with the definition's, cell by cell, under each view the declaration names (k-rust's kink-flow README, "Comparing implementations"). A case added to the corpus is compared with no other edit.
The verdicts are `kink.lock`'s `[compare]` section, so a moved verdict or first difference shows in review.

The Rust emulator's CPU is the one other implementation today: `krs/sm83-rust-cpu` runs `lib/crates/cpu` over the definition's memory model, from the start the definition's initializer builds for the program (kink gives it, so there is one statement of the boot state), and answers by the protocol `kink-implementation/1`: `stopped` or `final` with its end's cells, or `could-not-apply` with a reason.
It states every cell but the computation (`k`, the definition's internals) and `W` and `Z`, which the Rust CPU does not have.
The adapter locks on an unused opcode, is stuck on `STOP`, and idles one machine cycle per halted boundary, stopping where the corpus's `[stop]` does, as the definition does.
It sets a scheduled case's requests in `IF` between two of the Rust CPU's steps, and a step executes an instruction and then dispatches: where a request comes due inside an instruction or a dispatch, or at a boundary where `IME` is set, it cannot deliver the request where the definition does, and answers `could-not-apply`, naming the request.

The bus log (`read`, `write`, `idle` events) is compared under a view, a function of the cell's value applied to both sides (`krs/sm83-views`).
`WritesView` keeps only the writes; `AccessesView` keeps the reads and writes in order and drops idle cycles; no view keeps every event.
The Rust CPU performs an instruction's accesses at once, with no internal cycles, so its bus is compared under `writes` (`WritesView`, required) and `accesses` (`AccessesView`, reported).
At k-rust `8e938d911` (2026-11-11), under `writes` 37 of the 39 cases are equal and none differs; the other two are `could-not-apply`, the two whose requests come due where the Rust CPU cannot deliver them: `dispatch-request` (a request inside a dispatch) and `if-read-request` (a request read inside an instruction).
Under `accesses` the seven cases that dispatch an interrupt (`interrupt`, `interrupt-sources`, `joypad-int`, `halt-ime`, `ie-push-cancel`, `ram-code`, `halt-wake-ime`) differ, each first at its first dispatch, where the definition reads the opcode at `PC` and discards it and the Rust CPU pushes at once.
The definition's order is the one SameBoy, Gambatte and mooneye-gb share (`krs/sm83/notes/interrupt-sources.md`); Pan Docs and mooneye's hardware-verified `intr_timing` give only the five cycles, so no evidence here decides the order of accesses within them. The Rust CPU's order is that emulator's difference, not evidence against the definition's.
The Rust CPU's interrupt logic accesses `IE` and `IF` without bus cycles (`memory::Bus::read_interrupt_register`), so those accesses are not logged.

## Running

```sh
bun run compare             # krs kink compare, after building the ROMs and sm83-rust-cpu
bun run compare --locked    # the same, checking kink.lock's [compare] section
```

Only a required view decides the exit status: `different` exits 1. The adapter's answers are cached under its binary's digest, and every run compares them again with the definition as it is then.

## Cases

The 17 seed cases are the programs of the first mutation run (`draft-personal/observer-design/measurements/kill-power.md`), rewritten as cartridges.
Where those programs preset RAM, `IE` or `IF`, each case now writes the value in its prologue.
They start from the boot state, not from zeroed registers.

Three cover what the first `krs kink results` showed no case reaching: `ram-code` runs a routine from WRAM (fetches, a CB fetch, the HALT bug and a dispatch all reading RAM), `halt-ime` executes HALT with IME set and a request pending, and `ie-push-cancel` cancels a dispatch by pushing onto IE.
Five cover what `krs kink coverage` showed open at k-rust 86eafc770: `halt-mode` and `halt-mode-ime` enter `HALT` mode with IME clear and set, `interrupt-sources` requests STAT and serial together (priority, and the two vectors no case reached), `rom-write` writes into the ROM region, and `stop` executes `STOP`. `daa-flags` gained the four `DAA` inputs whose branch no case told apart: above `$99` with no carry, and a low digit above 9 with no half-carry, after an addition and after a subtraction.

Seven sweeps close the syntax family: at k-rust 86eafc770 `krs kink coverage` showed 82 `k::bits!` field values no case executed, and each sweep executes every encoding of the decode families those values belong to, not only the missing values, so it does not depend on kink's depth.
`cb-sweep` runs all 256 CB opcodes, `rst-sweep` the 8 `RST` vectors, `cc-sweep` `JR`, `JP`, `CALL` and `RET` on each condition in all four states of Z and C, `ld-sweep` the 63 `LD r, r'`, `alu-sweep` the 8 ALU operations on the 8 operands and an immediate, `incdec-sweep` `INC` and `DEC` on the 8 operands and the 4 pairs, and `word-sweep` `LD rr, nn`, `ADD HL, rr`, `LD (rr), A` and `LD A, (rr)`.
Each pushes or stores every result, from inputs chosen so that a wrong register, bit index, condition or vector gives a different end; its header says how.

Four scheduled cases close what no cartridge can, the open items of `krs kink coverage` at k-rust 599bb8485: `halt-wake` and `halt-wake-ime` are woken from `HALT` mode by a timer request, with IME clear (the CPU resumes, the request stays in `IF`) and set (the CPU dispatches it). `dispatch-request` requests VBlank inside a timer dispatch, before `IF` is sampled, so VBlank is served, and `if-read-request` places two requests around a read of `IF`, one in the read's cycle and one in the next.

Three close situations `krs kink query` showed no case reaching at k-rust b1966f5c6, each named by Pan Docs, while `coverage` showed nothing open: `ei-di` runs `EI` then `DI` with a request pending, so `DI` runs with `IME` set and no interrupt is served; `halt-bug-operand` puts `LD A, n` after a bugged `HALT`, which reads its own opcode as its operand; `halt-bug-rst` puts `RST` after a bugged `HALT`, which pushes its own address and so runs twice.

`scripts/roms.ts` pads every cartridge with `$DD`, the lock opcode, so a jump outside a case's code locks at once instead of running on.

## What each case tests

A case's header states its purpose in prose; `krs/sm83-patterns` states the situations the corpus must reach, as `k::patterns!` clauses over the definition's runs, and `FLOW.toml` declares that package (`[[patterns]] krs = "foldiboy-sm83-patterns"`).
`bun run kink query` finds, for each pattern, a case whose run reaches it (`hit`), and records that case as the pattern's witness in `kink.lock` (`[query]`), as `[run]` records a mutation's killer: the next query asks the recorded witness first and keeps it while it still hits, and a pattern no case reaches is recorded `open`.
So a new situation is written as a pattern first: if a case already reaches it, the lock names that case and no case is written; if none does, the pattern is recorded open, which is the case to write.
`query --locked` exits 1 when the query would change `[query]`: a witness deleted or no longer reaching its pattern moves the witness to another case that does, or the pattern to `open`.
`coverage` asks what the definition needs (each rule fired, each guard outcome shown); a pattern asks for a combination no single rule states, such as the HALT bug followed by a two-byte opcode, or an order along a run, such as a request that comes due inside a dispatch.

A clause is a chain of steps (k-rust's language reference, "Chain patterns"). Each step is a set of states written as a claim's subject, and a cell it does not name is unconstrained; `a => b` holds when a later state of the same run matches `b` under the same binders, and `unless(u)` on the arrow excludes a state matching `u` between them.
The package is outside the definition, so editing a pattern re-asks that pattern alone and moves no verdict.
Patterns are KRS, as the claims and the views are, while mutations are K patches of `definition.k`: a statement about the definition's behaviour is written in KRS, and an edit of the definition is a patch of the K it emits, anchored by KRS item (k-rust's mutation-testing guide, "Why mutations are K patches, not Rust edits").

## Mutation testing

The corpus is also kink's corpus for the SM83 definition: `FLOW.toml` names the definition (`krs = { definition = "main" }`) and takes every `cases/*/case.gb` as a program.
`mutations/<topic>.toml` holds K patches of `definition.k`, each a misunderstanding a definition's author could have, anchored by the label of the rule or equation it edits, which is the KRS item's.
kink runs each case on the definition and on each mutant and compares their ends on every cell except `w` and `z`, which the definition declares unobservable.

| Topic | Mutations of |
| --- | --- |
| `instructions` | the instructions' rewrite rules: bus order, internal cycles, the stack, memory, interrupts |
| `functions` | the functions they apply: ALU results and flags, decoding, conditions, interrupt selection |
| `boot` | the boot state the cases start from |
| `unobservable` | controls that change only W or Z after their last read: each must survive (`expect = "survive"`, which `run` checks) |

`definition.k` is the definition's emission, which `krs kink` writes when the definition changes; it is not committed.

```sh
bun run kink check          # admit the definition and the cases, resolve every mutation
bun run kink coverage       # what the cases cover: rules, guard outcomes, syntax, condition flips
bun run kink run            # each mutation's status and its killer
bun run kink run --full     # every case against every mutation: every killer of each
bun run kink query          # each pattern of krs/sm83-patterns has a case that reaches it
```

`kink.lock` is kink's record of the corpus, one section per command, each rewritten by its command after a run that judged everything and checked by it with `--locked` (exit 1 when it would change, 3 when the clock stopped a run it rests on: not decided on this host):

- `[run]`: each mutation's status and, for a killed one, one killer. A run tries the recorded killer first and keeps it while it still kills, so adding a case moves no killer and runs only the survivors on it; a killer that stops killing is replaced, and the diff shows it.
- `[coverage]`: each open obligation (a rule no case fires, a guard outcome no case shows, a production or `k::bits!` field value no case reaches), each argued one with its standing, and each condition flip (`[[coverage.flip]]`, a mutant kink generates by negating a guard or toggling a bit of an encoding) with what it changes, its status and its killer.
- `[compare]`: each other implementation's verdict on each case under each view (below, "Comparing implementations").
- `[query]`: each pattern's status (`hit`, `open` or `undecided`) and, for a hit, its witness (above, "What each case tests").

`run --full` costs every case on every mutation, so it is for occasional corpus pruning, not routine runs.
`krs kink prune` (`bun run kink prune`) reports which cases are redundant: it runs `coverage`, `run --full` and `query --full` and keeps a case only for an obligation no other case discharges (a mutation it alone kills, a pattern it alone hits, a coverage item it alone covers, a flip it kills), or one only it might discharge. It writes nothing, and it is not run routinely, since its cost is `run --full`'s.
So a case's purpose is kept by prune only if an obligation states it: a case whose header names a situation no pattern states is reported redundant whenever other cases cover what it covers (`halt-wake` and `halt-wake-ime` cover the same rules, and only their patterns tell them apart). Write the pattern when the case is written.
Under the Z3 profile kink caches results and outcomes (its solver queries are bounded by Z3 resource units, not the clock), so a rerun on an unchanged definition and corpus reuses them.
`[observe]` compares every run at each instruction boundary on `pc`, `ime` and `halted` as well as at its end, so a mutant whose run never ends is told apart where it first differs. `krs kink query` ends its runs at `[stop]` as `run` does. `fuzz` has no template to vary: kink varies only a program bound as `$PGM` text, and a case is a cartridge.

At k-rust 8e938d911, on the 39 cases (Z3 profile, 2026-11-11), every case is comparable.
Of 81 mutations, 72 are killed and the 9 `unobservable` controls survive, as they expect; none is unknown, and no mutant's run reaches the work bound (`[budget] work_factor`, kink's default of 5 times its program's baseline work).
Each of the 18 patterns of `krs/sm83-patterns` is a hit, and `[query]` names its witness; `query --full` finds 25 hitting pairs of 702, so 12 patterns are reached by one case only.
The three cases `query` found missing (`ei-di`, `halt-bug-operand`, `halt-bug-rst`) moved neither `[run]` nor `[coverage]`: they kill no mutation and close no coverage obligation, so their purpose is recorded only as the witnesses of their patterns in `[query]`, and `bun run compare` finds the Rust CPU equal on each.
`coverage`, for each family:

| Family | Covered | Open |
| --- | --- | --- |
| rules | 295 of 295 fire (claim helpers and the invariant are no obligation) | none |
| requirements | 196 of 202, 6 argued | none |
| syntax | 308 of 308 | none: the sweeps execute every `k::bits!` field value |
| condition flips | 38 of 38 distinguished | none |

The 6 argued requirements are the ROM side of the rules that read `memory` (`fetch`, `fetch_halt_bug`, `discard_fetch`, `fetch_cb`, `read_w`, `read_z`).
No case can reach them: `memory` never holds a ROM address, since a write into the ROM region stores nothing, and the rules carry their `!InRom` guards only so that krs's overlap check sees them disjoint from their ROM-side twins.
`FLOW.toml` argues each from the invariant `MemoryOutsideRom` (`krs/sm83/src/machine.rs`), and kink checks the argument: the invariant excludes the outcome at the rule's pattern, and it holds at every state of every run where those rules could apply (`checked-on-runs`).
It is checked on the corpus's runs, not proved; a refuted argument reopens its item and fails `coverage`.
