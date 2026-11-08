# Corpus

Programs every implementation of the Game Boy CPU runs, and the runner that compares them
(`draft-personal/corpus-contract`: decisions D-CASE-*, plan packages R7 and R8).

## A case

`cases/<id>/case.asm` is a cartridge in RGBDS syntax, and the case is the cartridge itself: `case.gb`, which `bun run roms` assembles beside it (ignored by git).
`krs run cases/<id>/case.gb`, `bun run kink` and `bun run compare` all start from that file.
It starts at `$0100` in the state the DMG boot ROM leaves, jumps to `Start`, and ends by locking the CPU with an unused opcode (`LOCK`, or `DUMP_AND_LOCK`, which first pushes every register and stores `SP` at `$C000`).
Two kinds of case end otherwise, as the definition does: `stop` ends stuck on `STOP`, which the CPU stage does not model, and a case that enters `HALT` mode with no interrupt requested never wakes (nothing at the CPU stage requests one), so its run ends at its second halted boundary (`FLOW.toml` `[stop]`, and the same stop in `bun run compare`).
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

`bun run compare` (`scripts/compare.ts`) compares every case's run on another implementation with the definition's run, cell by cell, with `krs compare`.
An implementation reports a run as a krs end document (`halt`, and every cell of the definition's configuration in the typed case encoding), in `build/ends/<id>.json`.

The Rust emulator's CPU is the one other system today: `krs/sm83-rust-cpu` runs `lib/crates/cpu` over the definition's memory model, from the state the definition's initializer builds from the case (`krs run --depth 0 --output end`, so there is one statement of the boot state), and writes its end document.
The adapter locks on an unused opcode, is stuck on `STOP`, and stops in `HALT` mode with no interrupt requested, as the definition does.

Every cell is compared; the bus log (`read`, `write`, `idle` events) is compared under a view, a function of the cell's value applied to both sides (`krs/sm83-views`).
`WritesView` keeps only the writes; `AccessesView` keeps the reads and writes in order and drops idle cycles; no view keeps every event.
The Rust CPU performs an instruction's accesses at once, with no internal cycles, so its bus is compared under `WritesView` (required equal) and `AccessesView` (reported).
Under `WritesView` every case is equal.
Under `AccessesView` the six cases that dispatch an interrupt (`interrupt`, `interrupt-sources`, `joypad-int`, `halt-ime`, `ie-push-cancel`, `ram-code`) differ, each first at its first dispatch, where the definition reads the opcode at `PC` and discards it and the Rust CPU pushes at once.
That bus order follows SameBoy (`krs/sm83/README.md`); Pan Docs gives only the five cycles, so which system matches hardware is open until a hardware run decides it.
The Rust CPU's interrupt logic accesses `IE` and `IF` without bus cycles (`memory::Bus::read_interrupt_register`), so those accesses are not logged.

## Running

```sh
bun run compare
```

It builds the cases' ROMs, writes each case's start with `krs run`, runs `sm83-rust-cpu` from those starts, and runs every case and view as one `krs compare --pairs` batch, printing each verdict and the first difference. It fails when a case is not equal under `WritesView`.

## Cases

The 17 seed cases are the programs of the first mutation run (`draft-personal/observer-design/measurements/kill-power.md`), rewritten as cartridges.
Where those programs preset RAM, `IE` or `IF`, each case now writes the value in its prologue.
They start from the boot state, not from zeroed registers.

Three cover what the first `krs kink results` showed no case reaching: `ram-code` runs a routine from WRAM (fetches, a CB fetch, the HALT bug and a dispatch all reading RAM), `halt-ime` executes HALT with IME set and a request pending, and `ie-push-cancel` cancels a dispatch by pushing onto IE.
Five cover what `krs kink coverage` showed open at k-rust 86eafc770: `halt-mode` and `halt-mode-ime` enter `HALT` mode with IME clear and set, `interrupt-sources` requests STAT and serial together (priority, and the two vectors no case reached), `rom-write` writes into the ROM region, and `stop` executes `STOP`. `daa-flags` gained the four `DAA` inputs whose branch no case told apart: above `$99` with no carry, and a low digit above 9 with no half-carry, after an addition and after a subtraction.

Seven sweeps close the syntax family: at k-rust 86eafc770 `krs kink coverage` showed 82 `k::bits!` field values no case executed, and each sweep executes every encoding of the decode families those values belong to, not only the missing values, so it does not depend on kink's depth.
`cb-sweep` runs all 256 CB opcodes, `rst-sweep` the 8 `RST` vectors, `cc-sweep` `JR`, `JP`, `CALL` and `RET` on each condition in all four states of Z and C, `ld-sweep` the 63 `LD r, r'`, `alu-sweep` the 8 ALU operations on the 8 operands and an immediate, `incdec-sweep` `INC` and `DEC` on the 8 operands and the 4 pairs, and `word-sweep` `LD rr, nn`, `ADD HL, rr`, `LD (rr), A` and `LD A, (rr)`.
Each pushes or stores every result, from inputs chosen so that a wrong register, bit index, condition or vector gives a different end; its header says how.

`scripts/roms.ts` pads every cartridge with `$DD`, the lock opcode, so a jump outside a case's code locks at once instead of running on.

## Mutation testing

The corpus is also kink's corpus for the SM83 definition: `FLOW.toml` names the definition (`krs = { definition = "main" }`) and takes every `cases/*/case.gb` as a program.
`mutations/<topic>.toml` holds K patches of `definition.k`, each a misunderstanding a definition's author could have, anchored by the label of the rule or equation it edits, which is the KRS item's.
kink runs each case on the definition and on each mutant and compares their ends on every cell except `w` and `z`, which the definition declares unobservable.

| Topic | Mutations of |
| --- | --- |
| `instructions` | the instructions' rewrite rules: bus order, internal cycles, the stack, memory, interrupts |
| `functions` | the functions they apply: ALU results and flags, decoding, conditions, interrupt selection |
| `boot` | the boot state the cases start from |
| `unobservable` | controls that change only W or Z after their last read: each must survive |

`definition.k` is the definition's emission, which `krs kink` writes when the definition changes; it is not committed.

```sh
bun run kink check          # admit the definition and the cases, resolve every mutation
bun run kink coverage       # what the cases cover: rules, guard outcomes, syntax, condition flips
bun run kink run            # each mutation's status and its killer
bun run kink run --full     # every case against every mutation: which cases are redundant
```

Two committed files are kink's record, each rewritten by the command after a run that judged everything, and checked by the same command with `--locked` (exit 1 when it would change):

- `verdicts.lock` (`run`): each mutation's status and, for a killed one, one killer. A run tries the recorded killer first and keeps it while it still kills, so adding a case moves no killer and runs only the survivors on it; a killer that stops killing is replaced, and the diff shows it.
- `coverage.lock` (`coverage`): each open obligation (a rule no case fires, a guard outcome no case shows, a production or `k::bits!` field value no case reaches) and each condition flip's status and killer.

`run --full` costs every case on every mutation, so it is for occasional corpus pruning, not routine runs.
Under the Z3 profile kink caches results and outcomes (its solver queries are bounded by Z3 resource units, not the clock), so a rerun on an unchanged definition and corpus reuses them.
`[observe]` compares every run at each instruction boundary on `pc`, `ime` and `halted` as well as at its end, so a mutant whose run never ends is told apart where it first differs. While the corpus has `[stop]` or `[observe]`, `krs kink query` and `fuzz` refuse it (their searches do not yet end or compare a path that way).

At k-rust 13819fe0f, on the 32 cases (Z3 profile, 2026-11-09), every case is comparable.
Of 81 mutations, 72 are killed and the 9 `unobservable` controls survive, as they must; none is unknown.
`coverage`, for each family:

| Family | Covered | Open |
| --- | --- | --- |
| rules | 291 of 292 fire (67 claim helpers not counted) | `boundary_wake`: waking from `HALT` needs a request while halted, which only a peripheral makes |
| requirements | 151 of 196 | 45, none a gap a case can close: 37 are outcomes kink records only when k-rust tries one equation of a function before another (an `if`/`else if` chain, or two complementary equations), 6 are the ROM side of rules that match `memory: { pc -> opcode }`, which never holds a ROM address, and 2 are `boundary_wake`'s |
| syntax | 307 of 307 | none: the sweeps execute every `k::bits!` field value |
| condition flips | 36 of 56 distinguished | the 20 others flip claim helpers, which no case runs; kink counts them |

The open requirements and the claim-helper flips are not cases to write: they are how `krs kink coverage` counts at this pin.
