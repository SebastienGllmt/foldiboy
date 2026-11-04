# Corpus

Programs every implementation of the Game Boy CPU runs, and the runner that compares them
(`draft-personal/corpus-contract`: decisions D-CASE-*, plan packages R7 and R8).

## A case

`cases/<id>/case.asm` is a cartridge in RGBDS syntax.
It starts at `$0100` in the state the DMG boot ROM leaves, jumps to `Start`, and ends by locking the CPU with an unused opcode (`LOCK`, or `DUMP_AND_LOCK`, which first pushes every register and stores `SP` at `$C000`).
`include/case.inc` holds the entry, the header space and those macros.

A case sets no register or memory directly: it writes what it needs in a prologue.
RAM starts empty, and a read of a byte the case never wrote ends the run as stuck. The definition and the Rust adapter treat it the same way.

## The machine

`machines/dmg-cpu-stage.toml` is the state every case starts from, as a krs case document of the SM83 definition's inputs.
Every value is the specification's own (`krs/sm83/src/boot.rs`); the runner adds the cartridge.
The Rust CPU starts from that same state, read from the definition's configuration, so there is one statement of the boot state.

## Systems and pairs

`corpus.toml` names the systems and the pairs to compare.

| Kind | System | End |
| --- | --- | --- |
| `krs` | The SM83 definition through krs-exec | `#locked`; another stuck head is `Stuck`; the step budget is `Bound` |
| `rust-cpu` | `lib/crates/cpu` over the definition's memory model (`krs/sm83/vectors/src/rust_cpu.rs`) | The adapter locks on an unused opcode and is stuck on `STOP`, as the definition is |
| `file` | An observation another system recorded: `<dir>/<case>.json` in the comparator's input encoding | As recorded |

Each run becomes an observation: the end class, the event log (`read`, `write`, `idle`) and the end state.
The comparator definition (`krs/sm83-compare`) compares two observations under a tier and normalizations:
- `Full`: every event.
- `Writes`: only writes.
- `CompressIdle`: runs of idle events become counts.

The Rust CPU performs an instruction's accesses at once, with no internal cycles, so it is compared at `Writes`.
Its interrupt logic's `IE` and `IF` accesses are not bus cycles (`memory::Bus::read_interrupt_register`), so they are not logged.

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
