# Corpus

Programs every implementation of the Game Boy CPU runs, and the runner that compares them
(`draft-personal/corpus-contract`: decisions D-CASE-*, plan packages R7 and R8).

## A case

`cases/<id>/case.asm` is a cartridge in RGBDS syntax, and `cases/<id>/case.toml` is the case: a krs case document of the SM83 definition's inputs, the cartridge `build.sh` makes from it (`build/<id>.gb`) and the boot state.
`krs run --case`, `krs kink` and the corpus runner all read that one document.
It starts at `$0100` in the state the DMG boot ROM leaves, jumps to `Start`, and ends by locking the CPU with an unused opcode (`LOCK`, or `DUMP_AND_LOCK`, which first pushes every register and stores `SP` at `$C000`).
`include/case.inc` holds the entry, the header space and those macros.

A case sets no register or memory directly: it writes what it needs in a prologue.
RAM starts empty, and a read of a byte the case never wrote ends the run as stuck. The definition and the Rust adapter treat it the same way.

## The machine

Every case starts on a DMG after its boot ROM, in the CPU stage: its inputs other than `rom` are the same in every case, applications of the specification's own boot functions (`krs/sm83/src/boot.rs`).
The Rust CPU starts from the state the definition's initializer builds from the case, read from its configuration, so there is one statement of the boot state.

## Comparing implementations

`run.sh` compares every case's run on another implementation with the definition's run, cell by cell, with `krs compare`.
An implementation reports a run as a krs end document (`halt`, and every cell of the definition's configuration in the typed case encoding), in `build/ends/<system>/<id>.json`.

The Rust emulator's CPU is the one other system today: `krs/sm83-rust-cpu` runs `lib/crates/cpu` over the definition's memory model, from the state the definition's initializer builds from the case (so there is one statement of the boot state), and writes its end document.
The adapter locks on an unused opcode and is stuck on `STOP`, as the definition is.

Every cell is compared; the bus log (`read`, `write`, `idle` events) is compared under a view, a function of the cell's value applied to both sides (`krs/sm83-views`).
`WritesView` keeps only the writes; `AccessesView` keeps the reads and writes in order and drops idle cycles; no view keeps every event.
The Rust CPU performs an instruction's accesses at once, with no internal cycles, so its bus is compared under `WritesView` (required equal) and `AccessesView` (reported).
Under `WritesView` every case is equal.
Under `AccessesView` the five cases that dispatch an interrupt (`interrupt`, `joypad-int`, `halt-ime`, `ie-push-cancel`, `ram-code`) differ, each first at its first dispatch, where the definition reads the opcode at `PC` and discards it and the Rust CPU pushes at once.
That bus order follows SameBoy (`krs/sm83/README.md`); Pan Docs gives only the five cycles, so which system matches hardware is open until a hardware run decides it.
The Rust CPU's interrupt logic accesses `IE` and `IF` without bus cycles (`memory::Bus::read_interrupt_register`), so those accesses are not logged.

## Running

```sh
corpus/run.sh            # every case
corpus/run.sh loads alu  # some cases
```

`run.sh` builds the cases with RGBDS from nixpkgs (`build.sh`), emits the definition's IR (the Rust CPU's initial state), runs `sm83-rust-cpu`, runs `krs compare` for each system, case and view, and writes `results/corpus.json`: per case and system, the end and each view's verdict, with the first difference; and the Rust CPU's instructions and M-cycles. It fails when a case is not equal under `WritesView`.

## Cases

The 17 seed cases are the programs of the first mutation run (`draft-personal/observer-design/measurements/kill-power.md`), rewritten as cartridges.
Where those programs preset RAM, `IE` or `IF`, each case now writes the value in its prologue.
They start from the boot state, not from zeroed registers.

Three more cover what the first `krs kink results` showed no case reaching: `ram-code` runs a routine from WRAM (fetches, a CB fetch, the HALT bug and a dispatch all reading RAM), `halt-ime` executes HALT with IME set and a request pending, and `ie-push-cancel` cancels a dispatch by pushing onto IE.

`build.sh` pads every cartridge with `$DD`, the lock opcode, so a jump outside a case's code locks at once instead of running on.

## Mutation testing

The corpus is also kink's corpus for the SM83 definition: `FLOW.toml` names the definition (`krs = { definition = "main" }`) and takes every `cases/*/case.toml` as a program.
`mutations/<topic>.toml` holds K patches of `definition.k`, each a misunderstanding a definition's author could have, anchored by the label of the rule or equation it edits, which is the KRS item's.
kink runs each case on the definition and on each mutant and compares their ends on every cell except `w` and `z`, which the definition declares unobservable.

| Topic | Mutations of |
| --- | --- |
| `instructions` | the instructions' rewrite rules: bus order, internal cycles, the stack, memory, interrupts |
| `functions` | the functions they apply: ALU results and flags, decoding, conditions, interrupt selection |
| `boot` | the boot state the cases start from |
| `unobservable` | controls that change only W or Z after their last read: each must survive |

`definition.k` is the definition's emission; `krs kink` rewrites it when the definition changes, and `run-kink.sh emit --check` fails when it is stale.

```sh
corpus/run-kink.sh check          # admit the definition and the cases, resolve every mutation
corpus/run-kink.sh results        # each case's end and the rules it fires
corpus/run-kink.sh run --full     # each mutation's killers
```

`results/kink-summary.json` is the last full run (`run --full`): each mutation's status and every case that kills it; `results/kink-verdicts.txt` is kink's own rendering of it (`run-kink.sh report --output verdicts` reprints a stored run).
Under the Z3 profile kink caches results and outcomes (its solver queries are bounded by Z3 resource units, not the clock), so a rerun on an unchanged definition and corpus reuses them (2026-11-06, under load: 456 s cold, 124 s warm, the same verdicts).

At k-rust b06b9b869, on the 20 cases (Z3 profile, 2026-11-06): kink compares every cell but `w` and `z`; every case is comparable and ends at `#locked`.
Of 81 mutations, 72 are killed and the 9 `unobservable` controls survive, as they must; none is unknown. The verdicts and killers are those of the first run at d0cf74b53.
Of the definition's 361 rules and equations, 291 are fired by some case and 67 are claim helpers, which only claims apply; the other three are `decode_stop` (STOP is outside the CPU stage) and `WellFormed` and `WellFormedRegisters`, which nothing applies.
