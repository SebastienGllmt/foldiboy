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

## Systems and pairs

`corpus.toml` names the systems and the pairs to compare.

| Kind | System | End |
| --- | --- | --- |
| `krs` | The SM83 definition through krs-exec | `#locked`; another stuck head is `Stuck`; the step budget is `Bound` |
| `rust-cpu` | `lib/crates/cpu` over the definition's memory model (`krs/sm83/vectors/src/rust_cpu.rs`) | The adapter locks on an unused opcode and is stuck on `STOP`, as the definition is |
| `file` | An observation another system recorded: `<dir>/<case>.json` in the comparator's input encoding | As recorded |

Each run becomes an observation: the end class, the event log (`read`, `write`, `idle`) and the end state.
The comparator definition (`krs/sm83-compare`) compares two observations under a tier and normalizations. The normalization `CompressIdle` turns runs of idle events into counts.

Tiers: `Writes` keeps only the writes. `Accesses` keeps the reads and writes in order but drops idles. `Full` keeps every event.

The Rust CPU performs an instruction's accesses at once, with no internal cycles, so it is compared at `Writes` and `Accesses`.
At `Writes` every case is equal.
At `Accesses` the five cases that dispatch an interrupt (`interrupt`, `joypad-int`, `halt-ime`, `ie-push-cancel`, `ram-code`) differ, each by one read per dispatch and nothing else:
the definition's interrupt dispatch begins with a discarded opcode read at `PC`, which the Rust CPU does not perform.
That bus order follows SameBoy (`krs/sm83/README.md`); Pan Docs gives only the five cycles, so which system matches hardware is open until a hardware run decides it.
The Rust CPU's interrupt logic accesses `IE` and `IF` without bus cycles (`memory::Bus::read_interrupt_register`), so those accesses are not logged.

The comparator is interim: k-rust request FB-21 asks for the same comparison in k-rust. The observations are its input either way.

## Running

```sh
corpus/run.sh            # every case
corpus/run.sh loads alu  # some cases
```

`run.sh` builds the cases with RGBDS from nixpkgs (`build.sh`), emits both definitions' IR, builds the runner (`krs/sm83/vectors`, binary `sm83-corpus`) and writes:
- `results/corpus.json`: per case and system, the end class, the event count and the costs (rewrite steps, instructions, M-cycles, wall time). Costs are reported, never compared. Per pair, the verdict.
- `build/observations/<system>/<case>.json`: each observation.

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

`results/kink-summary.json` is the last full run (`run --full`): each mutation's status and every case that kills it; `results/kink-verdicts.txt` is kink's own rendering of it (`--output verdicts`).


At k-rust d0cf74b53, on the 20 cases (Z3 profile, 2026-11-05): kink compares every cell but `w` and `z`; every case is comparable and ends at `#locked`.
Of 81 mutations, 72 are killed and the 9 `unobservable` controls survive, as they must; none is unknown.
Every rule and equation of the definition is fired by some case except `decode_stop` (STOP is outside the CPU stage), `WellFormed` and `WellFormedRegisters` (applied by no rule) and the claim helpers, which only claims apply.
