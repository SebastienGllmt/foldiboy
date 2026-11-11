# Native SM83 contracts

These Rust files are proof inputs, written in `k::claims!` over the existing machine.
`krs prove` supplies execution, simplification and SMT reasoning; the contracts add no execution rules, trusted axioms, custom runner or Lean code.
The native spec module `SM83-CONTRACTS-SPEC` imports the individual modules and selects the whole suite.

## Sources and justification

The expected results are specified independently of the implementation's `AluResult`, `AluFlags`, `UnaryResult`, `UnaryFlags`, `AddHlFlags`, `Holds`, `FlagByte`, `ByteFlags`, operand tables and arithmetic helpers.
Flags are written as the `Flags { z, n, h, c }` constructor: a claim binds the incoming fields it reads (`f: Flags { c: carry, .. }`), states each outgoing field as a Boolean expression, and leaves `f` out where the flags are unchanged.
Using builtin integer arithmetic, bitwise operations, map updates and conditional terms is intentional.
Repeated decision tables and long formulas use `Expected*` functions beside the claims: decoder field tables in `decode.rs`, DAA corrections in `instructions.rs`, the pushed flag byte in `memory.rs`, and interrupt priority in `boundaries.rs`.
These functions must remain independent of the implementation helpers; sharing an implementation table with an expected result would hide errors in that table.
The decoder tables use root `match` bodies and interrupt priority uses root `if` bodies, which KRS turns into defining equations.
The DAA formulas use short scalar conditional expressions inside named helpers.
KRS currently rejects function declarations inside spec modules, so the expectation helpers are declared outside those modules and emitted into `definition.k`; they add no machine transitions and are called only by the contracts.
The DAA helpers retain scalar `kequal::ite` bodies because the reviewed prover left their equivalent branching-body versions unresolved on symbolic register inputs.
The decoder tables retain the previous expressions' fallback values, while the claims restrict their indices to the corresponding opcode fields.

| Reference | Role |
| --- | --- |
| [RGBDS gbz80(7)](https://github.com/gbdev/rgbds/blob/dfe76a12c167aa0bc03a4cc9c30d534cbc45e2f1/man/gbz80.7) | Instruction meaning, flags, lengths, conditions and cycle counts. The instruction heading corresponding to a contract's mnemonic supplies its ordinary behavior specification. |
| [Pan Docs CPU Instruction Set](https://github.com/gbdev/pandocs/blob/abeda621759a8e2b322b368eacf4d31ec48eb4df/src/CPU_Instruction_Set.md) | Encoding families, register/condition field order, immediate width and little-endian representation. |
| [gbops DMG opcode data](https://github.com/izik1/gbops/blob/90b9bf296aed373335a0abbef0a0794a919b9f2c/dmgops.json) | Cross-check for opcode bytes and ordered fetch/read/write/internal cycles. It is reference data, not hardware measurements. |
| [Pan Docs Interrupts](https://github.com/gbdev/pandocs/blob/abeda621759a8e2b322b368eacf4d31ec48eb4df/src/Interrupts.md) and [HALT](https://github.com/gbdev/pandocs/blob/abeda621759a8e2b322b368eacf4d31ec48eb4df/src/halt.md) | Delayed EI, DI, interrupt priority/acknowledgement, HALT entry/wakeup and the HALT bug. |
| [mooneye `interrupts/ie_push.s`](https://github.com/Gekkio/mooneye-test-suite/blob/31510e12eea6286d36eea060a6adde755e1067aa/acceptance/interrupts/ie_push.s) | Hardware-verified upper-push cancellation and lower-push sampling cases, rounds 1 and 3. |
| [SameSuite `ei_delay_halt.asm`](https://github.com/LIJI32/SameSuite/blob/f15645fb049a47ea235f6d2c9a033e72d8087901/interrupt/ei_delay_halt.asm) | EI/HALT with prebuffered VBlank and STAT requests; the timer-arrival portion is outside this model. |

The five-cycle dispatch bus order follows the existing model's SameBoy-based interpretation; Pan Docs establishes the total cycle count but does not alone establish that precise bus log.
An external specification justifies a contract; proving the contract checks the implementation against that statement, not the authority or completeness of the specification itself.

## Numeric notation

Numbers are written according to their role, rather than giving every repeated value a new name.

| Role | Convention | Examples |
| --- | --- | --- |
| Memory-mapped register identity | Existing architectural constants | `IE` (`0xFFFF`), `IF` (`0xFF0F`) |
| Opcode or byte value | Two-digit hex | `0xFB` (EI), `0x76` (HALT), `0x06` (LD B,d8) |
| Flag weight or byte mask | Hex | Z/N/H/C use `0x80`/`0x40`/`0x20`/`0x10` in the `PUSH AF`/`POP AF` byte; a byte mask is `0xFF` |
| Interrupt bit set | Binary, grouped into nibbles | `0b0001_1111` for the five interrupt bits; `0b0000_0100` for timer |
| Fixed address or interrupt vector | Four-digit hex | `0x0100`, `0x0200`, `0x0040`, `0x0058` |
| Arithmetic radix or modulus | Hex power of two | `0x10` for nibbles, `0x100` for bytes, `0x1000` for the bit-11 carry boundary, `0x1_0000` for words |
| BCD digit boundary and corrections | Hex | `0x99`, `0x06`, `0x60` |
| Counts, bit indices and small offsets | Decimal | `pc + 2`, `bit` alternatives `0` through `7`, `delta == -1` |

The same numeric value can have different roles: the byte radix is `0x100`, while a fixture address is `0x0100`.
The contracts reuse only the existing IE/IF address constants, not implementation result/flag helpers or decoder tables.

Some arithmetic offsets are representations of wrapping operations.
Adding `0xFFFF` or `0xFFFE` before reducing modulo `0x1_0000` expresses a word decrement by one or two.
Adding `0x200` before a byte subtraction's `% 0x100` keeps the intermediate nonnegative under the stated byte bounds.
Those expressions retain their existing arithmetic shape in this notation cleanup.

Constants and symbolic inputs serve different purposes.
The IE-push fixtures deliberately use `PC = 0x0200, SP = 0x0000` and `PC = 0x0235, SP = 0x0001`: the pushed address bytes and their timing determine the interrupt behavior.
Their source comments explain those choices.
The EI/NOP/DI sequence fixtures' `0x0100` starting address could instead become a symbolic input during a coverage extension; doing so broadens the claim and is separate from changing notation.
The notation cleanup was checked against the reviewed revision: `krs check` passes, and emitted `definition.k` and `spec.k` are byte-identical to the pre-cleanup versions.

## Inventory

Each row identifies a family of native claims; individual names and preconditions are in the linked source.
All claimed bus logs append to an arbitrary existing log.
Unmodified cells are framed, including unrelated registers and memory.
Temporary `w`/`z` cells are exposed where execution changes them, so they are not accidentally asserted unchanged.

| Source / names | Property and reason for inclusion |
| --- | --- |
| [../claims.rs](../claims.rs), `add_hl_de` | Full 16-bit ADD HL,DE arithmetic and flags; retains the original dogfooding regression. |
| [decode.rs](decode.rs), `contract_decode_*` | Independent opcode-layout check, using symbolic bit-pattern conditions for 23 regular families and explicit cases for singleton and unsupported encodings over all 256 base and 256 CB byte values. Covers every encoding, not instruction execution. Twelve base-byte alternatives check the model's unsupported-opcode policy, including STOP, rather than claiming hardware support. |
| [instructions.rs](instructions.rs), `contract_nop` | NOP preserves the entire state after fetch. |
| `contract_ld8_*`, `contract_ld16_*`, `contract_ld_sp_hl` | Register, immediate and [HL] data paths; little-endian assembly, PC advancement, writes and flags preservation. These establish transport independently of arithmetic. |
| `contract_alu_*` | ADD/ADC/SUB/SBC/AND/XOR/OR/CP over register, immediate and [HL] operands; result, every flag, preservation and read timing. Register sources include A, covering accumulator aliasing. |
| `contract_unary_*` | INC/DEC; CB rotates/shifts/SWAP; accumulator rotates, DAA, CPL, SCF and CCF. Distinguishes accumulator rotations' forced Z=0 from CB rotations' result-dependent Z. Covers register and [HL] destinations where encodable. |
| `contract_bit_*` | BIT/RES/SET over all bit positions 0..7, register and [HL] paths; preserved flags and read/modify/write order. |
| `contract_inc16_*`, `contract_dec16_*` | Wrapping register-pair/SP arithmetic, unchanged flags and internal cycle. |
| `contract_add_hl_*` | All register-pair and SP sources, including source=HL aliasing; 16-bit wrap, Z preservation, N clearing and carries from bits 11 and 15. |
| `contract_jump_hl`, `contract_di`, `contract_ei` | Direct register jump and immediate/delayed interrupt-control effects. |
| [memory.rs](memory.rs), `contract_load_a_*`, `contract_store_a_*` | BC/DE, HL+/HL-, high-C, high-immediate and absolute addressing; address formation, pointer updates, data transport and ordered bus effects. |
| `contract_store_sp` | LD [d16],SP reads the address first and writes low then high, including address wrap. |
| `contract_push_*`, `contract_pop_*` | Stack byte order, SP wrap, preservation and POP AF's low-nibble mask. |
| `contract_sp_offset_*` | ADD SP,e8 and LD HL,SP+e8: signed result, unsigned low-byte carry flags and distinct internal-cycle counts. |
| [control.rs](control.rs), `contract_jumprelative_*`, `contract_jump_*`, `contract_call_*` | Taken/untaken outcomes for NZ/Z/NC/C and unconditional taken forms; consumed operands, signed offsets, PC wrap, stack writes and cycle differences. |
| `contract_return*`, `contract_restart` | RET/conditional RET/RETI and all eight RST vectors; return-address order, SP changes, conditional idle cycle and immediate RETI enable. |
| [fetch.rs](fetch.rs), `contract_fetch_*` | Fetch-to-finish composition for NOP, all seven LD r8,d8 encodings, all four LD r16,d16 encodings, and CB RLC A. These test opcode/operand reads and execution together; they are not end-to-end coverage of every opcode. |
| `contract_halt_bug_*` | The next fetch consumes the bug flag without incrementing PC; LD B,d8 rereads its opcode as the operand. |
| `contract_boundary_ld_hl_immediate` | One complete arbitration-to-finish instruction, with ordinary running state and no requested interrupt. |
| [boundaries.rs](boundaries.rs), `contract_halt_*` | HALT entry, idle, wake without dispatch and bug activation; prebuffered VBlank/STAT return-PC behavior. |
| `contract_boundary_*` | Arbitration selects dispatch using old IME, or fetch while applying delayed EI. These targets deliberately stop at the selected action. |
| `contract_dispatch_running`, `contract_dispatch_wake` | Full five-cycle dispatch, priority, IF acknowledgement, stack writes and IME clearing; ordinary cases exclude stack writes to IE/IF. |
| `contract_dispatch_ie_*` | Separate hardware-derived cases for stack aliasing IE; cancellation and late writes must not be hidden by ordinary-case exclusions. |
| `contract_ei_then_*` | EI/NOP, EI/DI and prebuffered EI/HALT sequences; one-step enable delay, DI cancellation and HALT return PC. |

The instruction schemas sometimes admit more typed register combinations than the ISA encodes; their actual encodings are a subset, checked separately by the decoder contracts.
Finite condition alternatives are explicit native KRS cases; every alternative must prove before the family is proven.
RST retains its eight explicit opcode alternatives: its symbolic arithmetic target remains beyond the current generated Lean closer, while the explicit form proves.
The six `contract_bit_*` claims match `bit @ (0 | 1 | 2 | 3 | 4 | 5 | 6 | 7)`.
These alternatives exhaust the existing `0 <= bit && bit < 8` premise and correspond to the three-bit opcode field.
Their original byte/flag premises, arithmetic postconditions, arbitrary register targets, memory and frames are retained.

The decoder cleanup task list and acceptance criteria are recorded in [sm83-decoder-cleanup.toml](../../../../machine-docs/sm83-decoder-cleanup.toml).
Its byte-domain audit compares every migrated family with its previous explicit list over all 256 bytes.
Expected field indices explicitly include their width, such as `(opcode / 0x10) % 4`, even where the family bounds already imply that width.
The audit also checks that these indices equal the previous quotient at every permitted byte; all other postcondition structure and the CPU semantics are unchanged.

## Scope and exclusions

- Decoded contracts start after opcode fetch and interrupt arbitration; their cycle logs contain only the remaining cycles.
- Fetch contracts start at `Micro::Fetch`; only the explicitly named `Step::Boundary` contracts include arbitration.
  A contract from `#boundary` ends at the next `#boundary`, the first state of the run its postcondition matches; the two-instruction EI sequences use concrete fixtures whose first boundary differs from their postcondition.
- Operand/register bounds are explicit where used; every `Flags` value is a valid flag state, so incoming flags need no premise.
  A written address must already be in the memory map (the write rule requires it).
  Each claim states only the premises it needs, spelled with `IsByte` and `IsWord` ([../well_formed.rs](../well_formed.rs)): fewer premises make a stronger statement, and a premise over the whole register map does not give `IsByte(r[src])` for a symbolic register `src` (the prover does not split `Reg8` into its seven constructors), so such a claim is stuck at its first step (measured 2026-11-04, `contract-alu-add-register`).
- Memory is the current flat map, where a write stores `Latch(addr, value)`: the byte with `IF`'s bits 7 to 5 set when `addr` is `IF`, and the byte itself elsewhere (`../machine.rs`). A contract that writes at a symbolic address states the stored byte as `Latch(addr, value)`, so it holds at `IF` too; reads need no such term, since `memory` holds what a read returns. No cartridge, other I/O register, timer, DMA, PPU, joypad, STOP execution or peripheral-driven interrupt arrival is established here.
- Every contract starts with `requests: []`: the rest of the console makes no interrupt request while it runs (the CPU stage's `REQUESTS` input, `../interrupt.rs`). A request that arrives partway through an instruction or a dispatch is covered by the corpus's `scheduled/` cases, not by a contract.
- The ordinary dispatch contracts exclude writes to IE/IF. IE alias cases are separately specified; writes onto IF and arbitrary asynchronous changes need further contracts and source qualification.
- Full decoder coverage plus decoded family contracts does not yet constitute a proof that every fetched instruction composes correctly. The fetch contracts above are the present composition coverage.
- `all_paths` is weak partial correctness. A proven result alone is not a termination theorem; inspect vacuity, trusted assumptions and claim applications separately.

## Running

### When to run them

The claims are not a routine check: the corpus with kink, SingleStepTests and mooneye are, and they cover every change to the definition in minutes.
A full run takes 15 to 25 minutes, so run `bun run claims` only when:

- a rule or function a claim states changes (`micro.rs`, `execute.rs`, `decode.rs`, `interrupt.rs`, the ALU and flag functions), or the configuration gains a cell, since every claim names the cells it fixes;
- before work that reads them, such as a k2lean translation.

Moving the k-rust pin is not a reason by itself; when a pin changes the prover, run them once to see whether a verdict moved.
They stay because they state each instruction family for every input, where SingleStepTests samples 1,000 vectors per opcode, and because a k2lean translation can start from them.

### How to run them

From `krs/sm83/`, using the CLI build and library environment described in the parent README:

```bash
"$KRS_BIN" prove --package foldiboy-sm83 --definition main \
  --dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events \
  --spec-module SM83-CONTRACTS-SPEC --depth 32
```

For the six bit-index contracts:

```bash
"$KRS_BIN" prove --package foldiboy-sm83 --definition main \
  --dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events \
  --spec-module SM83-INSTRUCTIONS-SPEC --depth 32 \
  --claim contract-bit-resetbit-register --claim contract-bit-resetbit-indirect \
  --claim contract-bit-setbit-register --claim contract-bit-setbit-indirect \
  --claim contract-bit-testbit-register --claim contract-bit-testbit-indirect
```

For only the decoder contracts, use `--spec-module SM83-DECODE-SPEC --depth 0`.
For an individual obligation, select its module and add `--claim contract-ld8-register` (for example).
Depth-zero decoder proofs check normalization against the expected decoded term, before any instruction transition.

To retain machine-readable diagnostics locally, append `--output json > results/claims.json` after creating `results/`.
That directory is ignored; saved output is not a build input or a substitute for rerunning the command.
On Linux with a user systemd manager, prefix a command with `systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0 env CARGO_BUILD_JOBS=1` to cap its memory, including collector builds and child processes.
An external resource limit produces no proof or refutation; retain it as an unqualified run.

## Lean decoder check

KRS claims remain the authored proof inputs.
`krust kcompile` supplies KORE and runtime metadata; `k2lean gen` translates the semantics and states its equation obligations; `k2lean claims` translates the selected contracts and attempts their proofs.
`lake build` checks those proofs and their generated axiom guards.
The completed decoder run must have no open equation obligations or open claims.
This workflow validates decoder contracts, not execution of every instruction or the entire contract suite.

Run from `krs/sm83/` with matching `KRS_BIN`, `KRUST_BIN`, `K2LEAN_BIN` and `K2LEAN_PRELUDE` (the absolute path of the k2lean Lake package directory).
The Lean toolchain is selected by the generated projects.
Build large projects sequentially and use the memory cap described above.

```bash
"$KRS_BIN" emit-k --package foldiboy-sm83 --definition main \
  --dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events --output results/emitted
"$KRUST_BIN" kcompile results/emitted/definition.k \
  --main-module SM83 --syntax-module SM83-SYNTAX \
  --output-directory results/lean/semantics
"$KRUST_BIN" kcompile results/emitted/spec.k \
  --main-module SM83-DECODE-SPEC --definition-module SM83 --for-proving \
  --output-directory results/lean/spec
"$K2LEAN_BIN" gen --kore results/lean/semantics/definition.kore \
  --runtime results/lean/semantics/runtime.json \
  --spec results/lean/spec/definition.kore \
  --out results/lean/library --name Sm83 --prelude "$K2LEAN_PRELUDE"
"$K2LEAN_BIN" claims --kore results/lean/semantics/definition.kore \
  --spec results/lean/spec/definition.kore \
  --out results/lean/claims --name Sm83Claims \
  --semantics Sm83 --semantics-path ../library \
  --kclaims-path "$K2LEAN_PRELUDE"
(cd results/lean/library && lake build Sm83 Sm83Test.Axioms)
(cd results/lean/claims && lake build Sm83Claims Sm83ClaimsTest)
```

`gen --spec` is required because the expectation functions are part of the contracts but unused by machine transitions.
The axiom guards permit only the standard Lean axioms and reject `sorry` or additional proof axioms.
A compact source condition may be proved by finite table-index cases internally; this does not change its quantified input domain.

## Proof evidence

The reviewed k-rust local-contract implementation is `492ac739a`; Foldiboy validation uses the containing merge `e50f28748`.
The upstream replay of all 181 original claims proved 172, left six unguided bit-index claims unresolved, and reached a 90-second external wall limit on three larger claims.
Our review independently reran eight unchanged instruction/sequence contracts and the original map reproducer successfully, including its certified negative control.

The six bit-index contracts now use exhaustive native input alternatives.
On 2026-10-28, `krs check` passed and all six adopted contracts proved: eight productive branches each, 48 total, with no reported vacuity, trusted reachability claims or claim applications.
The proof command completed in about 27 seconds after collection was built.
Validation used byte-identical Rust sources in an isolated copy, with only the two KRS facade dependency paths redirected to the reviewed checkout, while upstream frontend work continued separately.
The build and validation each used a 4 GiB cgroup cap, one Cargo job and zero swap; the validation group, including collection, peaked at approximately 2.4 GiB.
Together these runs provide proof evidence for 178 of 181 authored contracts.
The following table combines the upstream replay with that targeted validation; it is not a new full-suite replay.

On 2026-10-30, the `Expected*` helper refactor passed `krs check` and replayed all 46 decoder claims plus the DAA instruction claim at k-rust `d95e77fc9`.
Four temporary native equivalence claims also proved that the extracted DAA byte/flags and interrupt vector/mask expressions equal their previous inline forms under the existing byte and flag bounds.
The refactor preserves every claim subject and precondition and every existing definition sentence; it adds expectation functions and their defining equations.
This targeted validation does not reclassify the three open contracts or constitute a new full-suite replay.

The decoder cleanup was validated at k-rust `fe97012a4` on 2026-10-30.
It audits 24 regular families and converts 23; RST keeps its eight explicit cases because its compact arithmetic target does not close with the current generated Lean tactic.
The 23 migrated domains cover exactly the same 471 bytes as their previous lists; the complete base and CB pages still cover 512 bytes exactly once.
Eleven families now spell their field width explicitly with `% 4` or `% 8`; every helper argument is equal to the previous argument at every permitted byte, and all other postcondition structure is unchanged.
The emitted CPU `definition.k` is byte-identical.
The final `krs check` passes, and all 46 decoder claims prove at depth zero (64 leaves, formerly 512).
The common generated Lean library proves all 336 equation obligations with zero open statements; the final decoder module proves all 64 generated claims and passes its axiom guard.
Every decoder proof closes at the initial configuration without rule steps or claim applications.
This covers decoding only; the three older unresolved native contracts retain their previous status.

The cost comparison checks each audited family in both forms against the same library and tool revision.
Dependency builds and KORE compilation are outside the measured proof checks.
Each before/after pair uses the same protocol: a direct Lean module check, or a Lake proof build with dependencies already prepared; axiom guards are checked separately.
For the 23 converted families, one bounded run on the shared host measured:

| Metric | Explicit cases | Compact families |
| --- | ---: | ---: |
| Native isolated checks, total wall time | 5.65 s | 16.16 s |
| Lean checks, total wall time | 224.5 s | 140.2 s |
| Lean checks, total CPU time | 514.0 s | 123.8 s |
| Compiled claim modules, total size | 394.2 MB | 48.5 MB |
| Largest Lean scope memory peak | 2.53 GiB | 0.59 GiB |

Native symbolic reasoning is slower than evaluating concrete table entries; the measured increase is about 10.5 seconds across these isolated checks.
The cleanup accepts that cost in exchange for simpler source, much smaller Lean modules and lower total Lean proof cost, with unchanged CPU semantics.
These are single-run measurements, not stable performance guarantees; timings include process/import overhead and shared-host scheduling.
Raw comparisons and logs belong in ignored `results/decoder-cleanup-2026-10-30/`; the checked-in task record contains the acceptance summary rather than generated proofs or a custom runner.

| Module | Authored claims | Recorded proofs | Awaiting proof |
| --- | ---: | ---: | ---: |
| Original ADD HL,DE regression | 1 | 1 | 0 |
| Decoder | 46 | 46 | 0 |
| Decoded instructions | 76 | 76 | 0 |
| Addressing / stack / SP offsets | 17 | 17 | 0 |
| Control flow | 11 | 10 | 1 |
| Fetch composition | 16 | 16 | 0 |
| Boundaries / interrupts / HALT | 14 | 12 | 2 |

These counts concern grouped properties, not numbers of supported or correct opcodes.
The decoder's 46 authored claims cover 512 opcode bytes; regular families need not enumerate each byte as a proof branch.
The six bit-index execution claims retain their 48 explicit branches.
No successful claim in our targeted review reports vacuity, trusted reachability claims or claim applications.

On 2026-11-02, `bun run-claims.ts` replayed all 181 contracts with the krs CLI built from k-rust `1a937ef80` (`results/claims.json`, `results/claims-run.json`).
178 proved: the batch of 178 proved 177 in 18 minutes, and `contract-call-taken` proved alone in 2.6 minutes.
`contract-unary-daa-accumulator` failed with one stuck leaf at depth 4; it proved on 2026-10-30 at k-rust `d95e77fc9`. Bisected on 2026-11-09 (on this source, Foldiboy `778b465`): it proves at `b6434eb8a` and fails from `bb6b4ac90`, where the solver query of an implication stopped applying the definition's equations, so the final state's `unaryResult(unaryDaa, ..)` is never compared with the claim's `ExpectedDaaByte`. SingleStepTests' 1,000 `DAA` vectors pass, so the definition is not at fault; k-rust has it as FB-44.
`contract-dispatch-running` and `contract-dispatch-wake` each exceeded a 600-second budget without a verdict.
The machine's load average exceeded 500 during the run, so the durations are not cost measurements.

After the perpetual-boundary migration (foldiboy `6693b4d`), `bun run-claims.ts` replayed all 183 contracts with the same krs binary (SHA-256 `e1cffcb3…`).
180 proved, including the two added contracts (`contract-decode-illegal`, `contract-illegal-locks`) and the restated boundary and decoder-policy contracts; `contract-unary-daa-accumulator` still fails and the two dispatch contracts still exceed 600 seconds, as before the migration.

At k-rust `b06b9b869` (2026-11-06), `bun run-claims.ts` replayed all 183 contracts in one `krs prove --output jsonl --claim-timeout 600`, each solver query bounded by Z3 4.16.0 resource units (310,000, three doubling retries) rather than the clock: 182 proved in 18 minutes at load 20-40, and `contract-unary-daa-accumulator` still fails.
At k-rust `86eafc770` (2026-11-08, Z3 5.3.3), `bun run claims` gave the same verdicts: 182 proved in 12 minutes, and `contract-unary-daa-accumulator` fails.
At k-rust `13819fe0f` (2026-11-09), where a claim's status no longer depends on the order the search reaches its leaves, the verdicts are the same: 182 proved in 10 minutes.
At k-rust `599bb8485` (2026-11-09), with `MemoryOutsideRom` added to the definition, the verdicts are the same: 182 proved in 15 minutes on a busier host.
At k-rust `b1966f5c6` (2026-11-10), with `IF`'s write-side `Latch` and the solver bounded by Z3 resource units by default (CF-33), the verdicts are the same again: 182 proved in 14 minutes, `contract-unary-daa-accumulator` failing as FB-44 states.
At k-rust `2185855a7` (2026-11-11), which fixed FB-44 (a summary's body domain drops the conjuncts builtin hooks decide true, and a summary is specialized at a constructor argument's shape), all 183 contracts prove in 23 minutes at a 600-second bound, `contract-unary-daa-accumulator` included.
At k-rust `db7063a30` (2026-11-11), which marks constants in `definition.k` and adds chain patterns, all 183 still prove, in 20 minutes.
`contract-dispatch-running` and `contract-dispatch-wake` prove with `ExpectedInterruptVector` and `ExpectedInterruptMask` written as `if` chains: k-rust now admits those helpers' guards (`x / 2^k` is defined) to the prover's guarded reading (OT-20), so the single-expression workaround is gone.

Before the 2026-11-02 replay, `contract-call-taken`, `contract-dispatch-running` and `contract-dispatch-wake` each reached a 90-second external wall limit without a native verdict; all three prove at `b06b9b869`.
