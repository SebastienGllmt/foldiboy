# SM83 semantics in KRS

This crate defines the SM83 instruction set in KRS.
KRS is the only source of the semantics; the K it emits is a build output.

## Model

One instruction is one step, starting from `Micro::Fetch` and ending with an empty computation cell.
A step is a sequence of micro-operations.
`#run(n)` runs `n` boundary steps; each applies the `EI` delay, then dispatches an interrupt, fetches one instruction, or spends one cycle in `HALT` mode.
Each machine cycle of the instruction is one bus micro-operation, which appends one `Cycle` to the `bus` cell:
`read(addr, value)`, `write(addr, old, new)`, or `internal`.
A write records the byte it replaces, so a memory argument can check each write against the latest earlier access to the address.

A value read in one cycle and used in a later one passes through the internal registers `W` and `Z`, as on the hardware.
A value known when the instruction starts, such as a register pair being pushed, is computed by the instruction's rule and carried in the micro-operation.

The state is:

| Cell | Content |
| --- | --- |
| `regs` | the register file `A B C D E H L`, a map from `Register` to byte |
| `f` | the flag byte; Z, N, H, C in bits 7 to 4, bits 3 to 0 zero |
| `pc`, `sp` | 16-bit program counter and stack pointer |
| `ime` | the interrupt master enable |
| `eiPending` | set by `EI`; the next boundary sets `IME` after its interrupt check |
| `halted` | `HALT` mode |
| `haltBug` | the next opcode fetch leaves `PC` unchanged |
| `w`, `z` | internal temporary registers |
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

| File | Content |
| --- | --- |
| `machine.rs` | configuration, initializer, registers, bus cycles, register-pair and condition reads |
| `decode.rs` | the instruction syntax, opcode fields, operand tables, and `Decode`/`DecodeCb` |
| `execute.rs` | each instruction as its micro-operation sequence |
| `micro.rs` | micro-operations and their transitions |
| `interrupt.rs` | the boundary step: `EI` delay, `HALT` mode, interrupt dispatch |
| `alu.rs` | result and flag functions of the arithmetic and logic operations |
| `arith.rs` | fixed-width integer helpers |

## Conventions

- An opcode decodes into a typed `Instruction` through a table of bit templates (`k::bits!("00 yyy 100")` in `k::equations!`), whose fields index operand tables (`R8`, `Rp`, `Rp2`, `Cc`, `AluTable`, `Rot`, ...).
  An opcode no template matches decodes to `Instruction::Unimplemented`, which no rule executes.
  The index tables are `total` with `underspecified = ".."`: an index outside its field's range has no named value, which `krs check` would otherwise report.
- Functions are `#[k::function(total, internal)]` fns: a body is the one equation, or an `if` chain whose branches are its equations (`Signed`, `DaaHigh`, `InterruptVector`); a `;` declares a function that a table defines.
  None is program syntax, so none needs a `syntax` template.
- Terms use Rust operators, which KRS resolves to `Int` or `Bool` operations from the operands' sorts: the half-carry of an addition is `(a & 0x0F) + (b & 0x0F) > 0x0F`.
  K integers are unbounded, so nothing wraps implicitly; `Byte` and `Word` truncate.
  Opcodes, addresses and masks are written in hex, and the interrupt registers are the constants `IE` and `IF`.
- Collection cells and helper parameters declare element sorts: `Map<Reg8, Int>` for registers, `Map<Int, Int>` for memory, and `List<BusCycle>` for the bus log.
  KRS checks the element sorts through reads, writes and helper calls; these types do not impose byte or address bounds.
- Bus cycles append with builtin list notation (`[..log, Cycle::Read { addr, value }]`).
  The initializer keeps `list::empty()` because KRS does not yet accept `[]` there.
- Binder sorts in rules are inferred from the cells and constructor fields they match, so rules declare none.
- Transitions are readable `k::rules!` clauses whose computation-cell pattern is `[head, ..]`.
  A cell the rule reads but does not change is left out of the result.
  A cell written alone (`pc`) binds the cell to its own name.
  Rules on one head are distinguished by constructor patterns or by complementary guards (`g` and `!g`), so the overlap check can separate them.
- A result byte and its flags are separate functions (`AluResult`/`AluFlags`, `UnaryResult`/`UnaryFlags`).
- Unit constructors are written as qualified paths without parentheses (`Register::A`, `Micro::Idle`).
- Syntax tokens are distinct across sorts, so that text given to `krs run -c` parses one way; the harnesses pass typed values instead.

## Building

The crate depends on KRS by path (`Cargo.toml` and `vectors/Cargo.toml`), at a local k-rust checkout that is not yet published.
The last validated revision is `793cc0a3a`; path dependencies follow the checkout's current revision.
`KRS_ROOT` and `Z3_LIB` override the scripts' defaults.
The scripts find the test suites as `scripts/test.sh` does, under `FOLDIBOY_TEST_ROOT` (by default `roms/tests` beside the repository, reached through its logical path): `sm83/v1` (SingleStepTests) and `mooneye-test-suite`.
Build output goes to `target/` and `results/`, which are not tracked.

## Verification

`run-vectors.sh` checks the definition, emits its IR, builds the harness in `vectors/`, and runs every vector of the [SingleStepTests sm83](https://github.com/SingleStepTests/sm83) corpus.
The harnesses configure the definition with `krs-exec` values and read results by cell and constructor names (`Register::A`, `Cycle::Read`), never by emitted K symbols.
For each vector the harness compares the registers, flags, `PC`, `SP`, `IME`, the listed memory bytes, and the bus log.
An internal cycle is compared by kind only, because the corpus records the address and data lines left on the bus, which this model does not track.
The corpus is generated by an emulator; it is conformance evidence, not hardware or proof evidence.

`run-roms.sh` builds the mooneye acceptance ROMs that need only the CPU, RAM, `IE` and `IF` (with `wla-dx` from an ephemeral `nix-shell`) and runs each with `vectors/src/bin/sm83-rom.rs`.
The ROM fills `$0000-$7FFF` of a flat 64 KiB memory, the CPU starts in the post-boot state, and `LY` reads `$FF` so the mooneye harness skips its display code.
The tests' authors verified their expected results on hardware (the `Verified results` header of each mooneye source), so they are hardware evidence.

The migration from KRS `8299da60f` to `793cc0a3a` adds typed helper parameters and builtin list notation.
Its emitted `definition.k` is byte-identical to the unchanged source emitted by `793cc0a3a`.
The model keeps its hand layout: the current formatter still pads table labels before the colon and breaks some rows awkwardly.

Status at the last run:

- vectors: 498,000 of 500,000 pass; the failures are `HALT` and `STOP`.
  The corpus records `HALT` as a fetch plus two internal cycles with `PC + 1`, its generator's convention, which the model does not follow.
- mooneye: `ei_sequence`, `ei_timing`, `rapid_di_ei`, `reti_intr_timing` and `interrupts/ie_push` pass.
  `if_ie_registers` fails: it expects `IF` bits 7 to 5 to read as 1, an I/O register rule for the memory-map stage.
- SameSuite `interrupt/ei_delay_halt` (`SM83_DUMP=c000:24`): the VBlank and STAT entries match the hardware results (both return into the `HALT`); the third entry needs the timer.

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
