# SM83 semantics in KRS

This crate defines the SM83 instruction set in KRS.
KRS is the only source of the semantics; the K it emits is a build output.

## Model

The machine runs forever, as the hardware does, in boundary steps.
A boundary step starts at `#boundary`: it applies the `EI` delay, then dispatches an interrupt, fetches and executes one instruction, or spends one cycle in `HALT` mode, and queues the next `#boundary`.
A configuration whose computation cell is headed by `#boundary` is a state between two steps; observers compare those states.
Where to stop is the observer's choice (a claim's postcondition, a depth bound), not part of the state.
A step is a sequence of micro-operations.
An unused opcode locks the CPU: its step ends at `#locked`, after which nothing happens.
`STOP` decodes to `stop`, which no rule executes: the model does not support it.

A cartridge starts from the state a hardware model's boot ROM hands over, which `src/boot.rs` states once (`DmgRegisters`, `DmgFlags`, `BootPc`, `BootSp`, `DmgIo`, and `CpuStageIo` with the CPU stage's two display stand-ins), the cartridge's first 32 KiB as the ROM (`RomRegion`), and RAM that holds no byte until the program writes one. `Boot(cartridge)` puts these together as the definition's one input, `start` (a `State`, `src/machine.rs`); SingleStepTests states a `State` field by field instead.
A cartridge's case file gives `start` as `Boot` applied with `{"$call": ..}` (`mooneye/mooneye.toml`, `../../corpus/cases/`).

The initializer takes the whole architectural state at a boundary: `REGS`, `F`, `PC`, `SP`, `IME`, `EI_PENDING`, `HALTED`, `HALT_BUG` and `MEMORY`; and the unobservable registers `W` and `Z` (the harnesses pass 0), so a run can test that their initial value changes nothing observed.
The bus log starts empty.
Each machine cycle of the instruction is one bus micro-operation, which appends one event to the `bus` cell:
`read(addr, value)`, `write(addr, value)`, or `idle` (a cycle with no bus access).
The events are the observation contract's vocabulary, the `CpuEvent` sort of `../sm83-events`, which every implementation compared with this one reports in.
A write does not record the byte it replaces: that is memory at the address before the write, and a consumer that needs it derives it.

A value read in one cycle and used in a later one passes through the internal registers `W` and `Z`, as on the hardware.
A value known when the instruction starts, such as a register pair being pushed, is computed by the instruction's rule and carried in the micro-operation.

The state is:

| Cell | Content |
| --- | --- |
| `regs` | the register file `A B C D E H L`, a map from `Register` to byte |
| `f` | the flags, one `Flags { z, n, h, c }` value of four Booleans (zero, subtract, half-carry, carry) |
| `pc`, `sp` | 16-bit program counter and stack pointer |
| `ime` | the interrupt master enable |
| `eiPending` | set by `EI`; the next boundary sets `IME` after its interrupt check |
| `halted` | `HALT` mode |
| `haltBug` | the next opcode fetch leaves `PC` unchanged |
| `w`, `z` | internal temporary registers, declared `#[k::cell(unobservable)]`: no comparison of runs reads them |
| `memory` | the address space, a map from address to byte |
| `bus` | the bus log since the start |

`memory` is a flat map: this crate does not yet model the Game Boy memory map, cartridge, or I/O registers.
A read or write of an address absent from `memory` gets stuck instead of inventing a value.
`IE` (`$FFFF`) and `IF` (`$FF0F`) are bytes of `memory`.
The CPU reads them over internal wires, not the bus, so interrupt checks add nothing to the bus log; dispatch acknowledges a request by updating `IF` in `memory` directly.

## Interrupts and `HALT`

The rules follow the DMG behaviour that the mooneye and SameSuite hardware tests pin down, with SameBoy as the reference where the tests are silent.

- The boundary checks `IE & IF & $1F` against `IME` as it was before applying a pending `EI`, so exactly one instruction runs between `EI` and a dispatch.
- Dispatch takes five cycles: a discarded opcode read at `PC`, two internal cycles, and the pushes of `PC`, high byte first.
  `IE` is sampled after the high-byte push and `IF` after both, so a high-byte push onto `IE` (at `SP = $0000`) can change or cancel the interrupt; a cancelled dispatch jumps to `$0000`.
  `IME` is cleared and the served `IF` bit acknowledged; the lowest requested bit is served, at `$40 + 8 * bit`.
- `HALT` with no interrupt requested enters `HALT` mode, which spends one internal cycle per boundary until one is requested, then dispatches if `IME` is set and resumes otherwise.
- `HALT` with an interrupt already requested does not halt: with `IME` set it leaves `PC` on itself, so the handler returns to the `HALT`; with `IME` clear it triggers the `HALT` bug, and the next opcode fetch does not advance `PC`.
- `EI` with `IME` already set has no effect, as `eiPending` is cleared by the dispatch that follows it.

## Source layout

Arithmetic and ALU definitions live in `../sm83-core` and are re-exported by this crate.

| File | Content |
| --- | --- |
| `machine.rs` | configuration, initializer, registers, bus cycles, register-pair and condition reads |
| `decode.rs` | the instruction syntax, opcode fields, operand tables, and `Decode`/`DecodeCb` |
| `execute.rs` | each instruction as its micro-operation sequence |
| `micro.rs` | micro-operations and their transitions |
| `interrupt.rs` | the boundary step: `EI` delay, `HALT` mode, interrupt dispatch |
| `claims.rs`, `claims/` | native proof contracts, composed spec modules, and [coverage/source inventory](src/claims/README.md) |
| `../sm83-core/src/alu.rs` | result and flag functions of the arithmetic and logic operations |
| `../sm83-core/src/arith.rs` | fixed-width integer helpers |

## Conventions

- An opcode decodes into a typed `Instruction` through a table of bit templates (`k::bits!("00 yyy 100")` in `k::equations!`), whose fields index operand tables (`R8`, `Rp`, `Rp2`, `Cc`, `AluTable`, `Rot`, ...).
  Every byte decodes: `STOP` to `stop` (unsupported) and the eleven unused opcodes to `#illegal(opcode)`, which locks the CPU; `Decode` has no `owise` equation.
  Decoder contracts use `k::bits!(opcode, "01 ___ ___")` conditions for regular families and retain explicit cases for singleton encodings and the irregular unsupported-opcode set.
  The named `HALT_OPCODE` exception excludes HALT from the register-load family; expected operand tables remain independent of the implementation tables.
  The index tables are `total` with `underspecified = ".."`: an index outside its field's range has no named value, which `krs check` would otherwise report.
- Functions are `#[k::function(total, internal)]` fns: a body is the one equation, or an `if` chain whose branches are its equations (`Signed`, `DaaHigh`, `InterruptVector`); a `;` declares a function that a table defines.
  None is program syntax, so none needs a `syntax` template.
- Terms use Rust operators, which KRS resolves to `Int` or `Bool` operations from the operands' sorts: the half-carry of an addition is `(a & 0x0F) + (b & 0x0F) > 0x0F`.
  K integers are unbounded, so nothing wraps implicitly; `Byte` and `Word` truncate.
  Opcodes, addresses and masks are written in hex, and the interrupt registers are the constants `IE` and `IF`.
- Collection cells and helper parameters declare element sorts: `Map<Reg8, Int>` for registers, `Map<Int, Int>` for memory, and `List<CpuEvent>` for the bus log.
  KRS checks the element sorts through reads, writes and helper calls; these types do not impose byte or address bounds.
  Read entries with `registers[Register::A]` or `memory[addr]`, and check membership with `memory.contains_key(addr)`.
  Write entries with `k::map![high -> hi, low -> lo, ..registers]`; entries apply in order over the original map, so the later entry wins if keys alias.
  These forms denote builtin map operations; reads of absent keys remain undefined.
- Bus cycles append with builtin list notation (`[..log, Event::Read { addr, value }]`).
  The initializer keeps `list::empty()` because KRS does not yet accept `[]` there.
- Binder sorts in rules are inferred from the cells and constructor fields they match, so rules declare none.
- Transitions are readable `k::rules!` clauses whose computation-cell pattern is `[head, ..]`.
  A cell the rule reads but does not change is left out of the result.
  A cell written alone (`pc`) binds the cell to its own name.
  Rules on one head are distinguished by constructor patterns or by complementary guards (`g` and `!g`), so the overlap check can separate them.
- A result byte and its flags are separate functions (`AluResult`/`AluFlags`, `UnaryResult`/`UnaryFlags`).
  A flag function returns a `Flags` value; a condition reads its fields (`Holds`).
  Only `PUSH AF` and `POP AF` see `F` as a byte: `FlagByte` packs Z, N, H, C into bits 7 to 4 with bits 3 to 0 zero, and `ByteFlags` unpacks a byte, ignoring bits 3 to 0, so an `F` with a non-zero low nibble is unrepresentable.
- Unit constructors are written as qualified paths without parentheses (`Register::A`, `Micro::Idle`).
- Syntax tokens are distinct across sorts, so that text given to `krs run -c` parses one way; the harnesses pass typed values instead.

## Building

The KRS crates and the scripts use k-rust from a pinned worktree, `/artifacts/worktrees/k-rust-foldiboy-pin`, a detached checkout of the commit recorded in `../K_RUST_PIN`, with its `krs` CLI built there (`cargo build -p krs-cli --bin krs --locked`).
Moving the pin is a deliberate change: check out the new commit there, rebuild, update `K_RUST_PIN`, and rerun the verification. The k-rust development checkout moves on its own; a `krs` binary and a `krs` crate from different commits disagree on the collector protocol.
`KRS_BIN` overrides the scripts' default binary. krs links the Z3 release its `Krs.lock` names (4.16.0), statically, so no Z3 library path is needed.
The scripts are `bun run` commands of the repository's `package.json` (`../../scripts/`). `scripts/krs.ts` runs krs with the root definition's selection, `--dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events`; direct KRS commands require the same selection.
The scripts find the test suites under `FOLDIBOY_TEST_ROOT` (by default `roms/tests` beside the repository, reached through its logical path): `sm83/v1` (SingleStepTests) and `mooneye-test-suite`.
Build output goes to `target/`, `results/` and the mooneye ROMs beside `mooneye/mooneye.toml`, none of them tracked.

## Tools and workflow

The same KRS semantics feed concrete execution and symbolic proofs.
Claims specify properties to check; they do not define additional CPU behavior.

| Tool or input | Purpose in this project |
| --- | --- |
| KRS configuration, functions and `k::rules!` | Define the machine state and SM83 transitions. |
| `krs check` | Check that the composed definition is well formed before executing or proving it. This does not establish SM83 correctness. |
| `krs emit-k` | Generate K and definition IR from the semantics. The Rust CPU adapter (`../sm83-rust-cpu`) builds a case's initial state from the IR through `krs-exec`. |
| `krs cases`, `krs run --case` | Run case documents and check each against its expectation: the upstream suites' cases and `mooneye/mooneye.toml`. |
| `bun run sst` | Run the upstream SingleStepTests SM83 corpus against the semantics. |
| `bun run mooneye` | Run upstream mooneye acceptance ROMs against the semantics. |
| `k::claims!` in [src/claims.rs](src/claims.rs) and [src/claims/](src/claims/README.md) | Specify CPU behavior as symbolic execution contracts over the KRS machine, so the native prover can check the instruction rules against them. |
| `bun run claims` | Run `krs prove` over the contract suite with a per-claim timeout and save its records to `results/claims.jsonl`. |
| `krs prove` | Attempt to establish those claims using k-rust’s native reachability prover through `krs-exec`, with Z3 for symbolic conditions. A failed or stuck proof requires investigation; it does not by itself identify a CPU bug. |
| `krust kcompile` | Compile emitted K semantics and a selected spec module to the KORE and runtime metadata consumed by k2lean. |
| `k2lean gen`, `k2lean claims` | Generate a Lean semantics library, its equation obligations, and proofs of the selected KRS contracts. `gen --spec` includes expectation helpers used only by claims. |
| `lake build` and generated axiom guards | Check generated Lean proofs and reject added axioms or `sorry`. Open statements are not proofs. |

The Lean decoder workflow and its proof scope are documented in the [claim inventory](src/claims/README.md#lean-decoder-check).

Use the upstream suites for compatibility evidence on concrete executions.
Use claims to state the CPU contracts that must hold across symbolic inputs, such as an instruction's result, preserved flags and bus effects.
Claims give the native prover an initial state, precondition and postcondition; it derives the intervening execution from the existing instruction rules.
We use this interface to automate proofs about those rules without writing their transition reasoning by hand in Lean.
Lean remains available for proofs that need explicit mathematical arguments, such as refinement between different machine models; claims are not a prerequisite for every Lean proof.
The `all_paths` claim uses weak partial correctness: every finite maximal execution from its permitted initial states must pass through the postcondition; termination is not established by this modality alone.
A proof applies only to the modeled behavior and the claim's preconditions; it does not validate the model against hardware.

### Commands

Run from `krs/sm83/`, with `KRS_BIN` pointing to the KRS CLI and its required shared libraries available.
All commands composing the CPU definition need `--dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events`.

Check the definition:

```bash
"$KRS_BIN" check --package foldiboy-sm83 --definition main \
  --dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events
```

Run the native contract suite (sources, justifications and coverage are in the [claim inventory](src/claims/README.md)):

```bash
"$KRS_BIN" prove --package foldiboy-sm83 --definition main \
  --dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events \
  --spec-module SM83-CONTRACTS-SPEC --depth 32
```

Success requires every selected claim to report `proven` and exit status 0.
`bun run claims` runs the same command with `--output jsonl` and `--claim-timeout 600`, saves krs's records to `results/claims.jsonl` and prints the summary.
krs proves each claim in a supervised worker: a claim that runs past the timeout gets the verdict `timeout`, and a solver crash costs only its claim (`crashed`); neither establishes anything about the claim.
Each solver query is bounded by Z3 resource units rather than the clock, so a verdict does not depend on the host's load.
The six bit-index contracts use exhaustive native cases over the three-bit opcode field.
The [claim inventory](src/claims/README.md#proof-evidence) records the reviewed proof evidence; at k-rust `b06b9b869` every contract proves except `contract-unary-daa-accumulator`.
Proof output is printed to the terminal; any saved diagnostic runs belong in ignored `results/`.

Run the concrete test suites:

```bash
bun run sst
bun run mooneye
```

## Upstream conformance suites

`bun run sst` (`scripts/singlestep.ts`) and `bun run mooneye` (`mooneye/mooneye.toml`) are adapters for the named upstream suites only.
Their scope is building and loading those suites, executing them, and reporting their prescribed checks.
Do not add project-authored test cases, proof workflows or general test infrastructure to them.
Changes should be limited to maintaining the suite integration as the build system, execution API or upstream suite changes.
Keep project-authored verification separate, such as the symbolic contracts in `src/claims.rs`.

`bun run sst` converts every vector of the [SingleStepTests sm83](https://github.com/SingleStepTests/sm83) corpus into a case document (`scripts/singlestep.ts`, which states the mapping), runs them with `krs cases`, and saves krs's records to `results/singlestep.jsonl`.
Each vector becomes a named case: its `start` is the vector's initial state, a `State`, and its expectation the final registers, flags, `PC`, `SP`, `IME`, `EI`'s pending state, memory and bus log, compared by `krs cases` cell by cell.
A case runs from its boundary to the next boundary rule: it stops before a boundary rule, at the second stop (the first is the initial state).
`IE` and `IF` read as zero when a vector does not list them (no vector lists both), so no interrupt is requested: the corpus does not model interrupts.
`F` is a `Flags` value built from the vector's byte.
An idle cycle is compared by kind only, because the corpus records the address and data lines left on the bus, which this model does not track.
`--wz W,Z` sets the unobservable registers' initial value; every vector must give the same verdict for any value.
The corpus is generated by an emulator; it is conformance evidence, not hardware or proof evidence.

`bun run mooneye` builds the mooneye acceptance ROMs that need only the CPU, RAM, `IE` and `IF` (`scripts/roms.ts`, with WLA-DX from nixpkgs) beside `mooneye/mooneye.toml`, and runs its cases with `krs cases`.
Each case starts from the ROM's cartridge as the boot ROM leaves it (`Boot`) (the CPU stage's `LY` reads `$FF`, so the mooneye harness skips its display code), and expects mooneye's pass registers, `B C D E H L` = 3 5 8 13 21 34, compared under the view `MooneyeResult` (`../sm83-views`).
The tests' authors verified their expected results on hardware (the `Verified results` header of each mooneye source), so they are hardware evidence.
A mooneye test ends by looping on itself, so each run is bounded at 100000 rewrite steps, past every ROM's result; `results/mooneye.jsonl` holds krs's record of each case (`krs-cases/2`).

The migration from KRS `8299da60f` to `793cc0a3a` adds typed helper parameters and builtin list notation.
Its emitted `definition.k` is byte-identical to the unchanged source emitted by `793cc0a3a`.
The model keeps its hand layout: the current formatter still pads table labels before the colon and breaks some rows awkwardly.

Status at the last run:

- vectors: 498,000 of 500,000 pass; the failures are `HALT` and `STOP`.
  The corpus records `HALT` as a fetch plus two internal cycles with `PC + 1`, its generator's convention, which the model does not follow.
- mooneye: `ei_sequence`, `ei_timing`, `rapid_di_ei`, `reti_intr_timing` and `interrupts/ie_push` pass.
  `if_ie_registers` fails: it expects `IF` bits 7 to 5 to read as 1, an I/O register rule for the memory-map stage.
- SameSuite `interrupt/ei_delay_halt` (last checked with the removed `sm83-rom` harness's `SM83_DUMP=c000:24`; not rerun since; `krs run --case` prints the memory to recheck it): the VBlank and STAT entries match the hardware results (both return into the `HALT`); the third entry needs the timer.

| Group | Opcodes | Status |
| --- | --- | --- |
| 8-bit loads between registers, `(HL)` and immediates | `LD r,r'`, `LD r,n` | done |
| loads through `A` | `LD (rr),A`, `LD A,(rr)`, `LD (HL±)`, `LDH`, `LD (C)`, `LD (nn)` | done |
| 16-bit loads and arithmetic | `LD rr,nn`, `INC rr`, `DEC rr`, `ADD HL,rr`, `LD (nn),SP`, `LD SP,HL`, `LD HL,SP+e`, `ADD SP,e` | done |
| 8-bit arithmetic and logic | `ALU A,r`, `ALU A,n`, `INC r`, `DEC r` | done |
| accumulator and flag operations | `RLCA`, `RRCA`, `RLA`, `RRA`, `DAA`, `CPL`, `SCF`, `CCF` | done |
| CB page | rotates, shifts, `SWAP`, `BIT`, `RES`, `SET` | done |
| jumps, calls and returns | `JR`, `JP`, `JP HL`, `CALL`, `RET`, `RETI`, `RST`, with conditions | done |
| stack | `PUSH rr`, `POP rr` | done |
| interrupts | `DI`, `EI`, `RETI`, dispatch | done |
| CPU control | `HALT`, `STOP` | `HALT` done (its bug untested: no peripheral-free test); `STOP` open, needs the joypad and `DIV` |
