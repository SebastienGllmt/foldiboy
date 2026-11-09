//! Architectural state, the bus, and the initial configuration.

use k::Term;
use k::builtin::{Bool, Bytes, Int, KSequence, List, Map, bytes};

use crate::alu::{FlagByte, FlagSet, Flags};
use crate::arith::Pair;
use crate::boot::{BootPc, BootSp, CpuStageIo, DmgFlags, DmgRegisters, RomRegion};
use crate::decode::{Cond, Condition, Pair16, RegPair};
use crate::interrupt::{IF, IF_UNUSED_BITS, InterruptRequest, Step};

pub use sm83_events::{CpuEvent, Event};

k::sorts! {
    Reg8;
}

/// The 8-bit registers addressed by instruction encodings.
///
/// `F` is not among them: no encoding names it as an 8-bit operand.
#[k::constructors(sort = Reg8, result = k::KResult)]
pub enum Register {
    #[k::syntax("a")]
    A,
    #[k::syntax("b")]
    B,
    #[k::syntax("c")]
    C,
    #[k::syntax("d")]
    D,
    #[k::syntax("e")]
    E,
    #[k::syntax("h")]
    H,
    #[k::syntax("l")]
    L,
}

#[k::configuration]
pub struct Machine {
    #[k::cell]
    pub k: KSequence,
    /// The 8-bit register file.
    #[k::cell]
    pub regs: Map<Reg8, Int>,
    /// The flag register: zero, subtract, half-carry and carry.
    #[k::cell]
    pub f: FlagSet,
    #[k::cell]
    pub pc: Int,
    #[k::cell]
    pub sp: Int,
    #[k::cell]
    pub ime: Bool,
    /// Set by `EI`: `IME` becomes set after the following instruction.
    #[k::cell(name = "eiPending")]
    pub ei_pending: Bool,
    /// `HALT` mode: each boundary spends one cycle until an interrupt is requested.
    #[k::cell]
    pub halted: Bool,
    /// Set by the `HALT` bug: the next opcode fetch does not advance `PC`.
    #[k::cell(name = "haltBug")]
    pub halt_bug: Bool,
    /// Internal temporary register, the high byte of `WZ`.
    ///
    /// Unobservable: every instruction that reads `W` writes it earlier in the
    /// same instruction, so its value at a boundary never reaches an event or an
    /// architectural cell. Tested by running SingleStepTests with two different
    /// initial values of `W` and `Z` (every vector starts at a boundary): every
    /// vector gives the same result (`bun run sst --wz W,Z`).
    #[k::cell(unobservable)]
    pub w: Int,
    /// Internal temporary register, the low byte of `WZ`. Unobservable for the
    /// same reason as `w`, tested the same way.
    #[k::cell(unobservable)]
    pub z: Int,
    /// The cartridge ROM: read-only, at addresses 0 up to its length, at most
    /// 32 KiB (so `IE`, `IF` and RAM are always in `memory`). Empty for
    /// a flat-RAM machine (SingleStepTests, the claims), where `memory` holds
    /// every address.
    #[k::cell]
    pub rom: Bytes,
    /// The end of the ROM region: the ROM's length, set by the initializer. The
    /// region test reads this Int, so a proof decides it by integer arithmetic
    /// and never puts a `Bytes` term in front of the solver.
    #[k::cell(name = "romEnd")]
    pub rom_end: Int,
    /// Every other address, as a map from address to byte. A read of an address
    /// the map does not hold is stuck: the model gives unwritten RAM no value.
    #[k::cell]
    pub memory: Map<Int, Int>,
    /// The event log: what the CPU showed outside itself since the start, one
    /// event per machine cycle (the observation contract's vocabulary, `sm83-events`).
    #[k::cell]
    pub bus: List<CpuEvent>,
    /// The CPU stage's input from the rest of the console: the interrupt requests it makes, in
    /// cycle order, each removed when `deliver_request` sets its bits in `IF`. The console stage
    /// replaces it with the peripherals.
    #[k::cell]
    pub requests: List<InterruptRequest>,
}

/// The initial state of a cartridge: the state a DMG's boot ROM leaves in the CPU stage (boot.rs).
impl Machine {
    #[cfg(not(whole_state))]
    #[k::init]
    pub fn init(
        #[k::config_var(name = "CARTRIDGE", main)] cartridge: Bytes,
        #[k::config_var(name = "REQUESTS")] requests: List<InterruptRequest>,
    ) -> Self {
        Self {
            k: k::k![Step::Boundary],
            regs: DmgRegisters(),
            f: DmgFlags(),
            pc: BootPc(),
            sp: BootSp(),
            ime: false,
            ei_pending: false,
            halted: false,
            halt_bug: false,
            memory: CpuStageIo(),
            w: 0,
            z: 0,
            rom_end: bytes::length(RomRegion(cartridge)),
            rom: RomRegion(cartridge),
            bus: k::builtin::list::empty(),
            requests,
        }
    }

    /// With `--cfg whole_state`, any boundary state starts a run: SingleStepTests' vectors.
    #[cfg(whole_state)]
    #[k::init]
    pub fn init(
        #[k::config_var(name = "REGS")] regs: Map<Reg8, Int>,
        #[k::config_var(name = "F")] f: FlagSet,
        #[k::config_var(name = "PC")] pc: Int,
        #[k::config_var(name = "SP")] sp: Int,
        #[k::config_var(name = "IME")] ime: Bool,
        #[k::config_var(name = "EI_PENDING")] ei_pending: Bool,
        #[k::config_var(name = "HALTED")] halted: Bool,
        #[k::config_var(name = "HALT_BUG")] halt_bug: Bool,
        #[k::config_var(name = "ROM")] rom: Bytes,
        #[k::config_var(name = "MEMORY")] memory: Map<Int, Int>,
        #[k::config_var(name = "W")] w: Int,
        #[k::config_var(name = "Z")] z: Int,
    ) -> Self {
        Self {
            k: k::k![Step::Boundary],
            regs, f, pc, sp, ime, ei_pending, halted, halt_bug, memory, w, z,
            rom_end: bytes::length(rom),
            rom,
            bus: k::builtin::list::empty(),
            // A vector is one instruction from a boundary, with no console around it.
            requests: k::builtin::list::empty(),
        }
    }
}

/// Whether `addr` is in the ROM region, 0 up to `rom_end`.
#[k::function(total, internal)]
pub fn InRom(rom_end: Term<Int>, addr: Term<Int>) -> Term<Bool> {
    0 <= addr && addr < rom_end
}

// An I/O register is not RAM: `memory` holds the byte a read of each address returns, and a write
// stores what the register then reads back (`Latch`). Every bus write outside the ROM goes
// through it. `IF` latches its five request bits, and its bits 7 to 5, which do not exist, read
// as 1 (Pan Docs; mooneye if_ie_registers), so a write to it stores them set. The masking is on
// the write side, not the read side, so a read is a plain lookup: the claims about reads and
// fetches from any address need no premise that the address is not `IF`. With
// `--cfg whole_state` (SingleStepTests) every address is RAM.

/// The byte `memory` holds at `addr` after a bus write of `value`.
#[cfg(not(whole_state))]
#[k::function(total, internal)]
pub fn Latch(addr: Term<Int>, value: Term<Int>) -> Term<Int> {
    if addr == IF { value | IF_UNUSED_BITS } else { value }
}

/// The byte `memory` holds at `addr` after a bus write of `value`: every address is RAM.
#[cfg(whole_state)]
#[k::function(total, internal)]
pub fn Latch(addr: Term<Int>, value: Term<Int>) -> Term<Int> {
    value
}

/// The value of the register pair `HL`.
#[k::function(total, internal)]
pub fn Hl(registers: Term<Map<Reg8, Int>>) -> Term<Int> {
    Pair(registers[Register::H], registers[Register::L])
}

/// Whether no address `memory` holds is in the ROM region: an invariant of every run from a
/// cartridge, since a write into the ROM region stores nothing. Only kink reads it, as the
/// invariant the corpus's argued ROM-side obligations rest on (corpus/FLOW.toml); no rule
/// applies it. It is written as its violation, so that whichever binding the map pattern
/// chooses, the value is the same.
#[k::function(total, internal)]
pub fn MemoryOutsideRom(memory: Term<Map<Int, Int>>, rom_end: Term<Int>) -> Term<Bool>;

k::equations! {
    for MemoryOutsideRom;
    #[k::requires(InRom(rom_end, addr))]
    memory_in_rom: (k::builtin::map![addr -> _, ..], rom_end) => false;
    #[k::equation(owise)]
    memory_outside_rom: (_, _) => true;
}

/// The value of a register pair.
#[k::function(total, internal)]
pub fn PairValue(
    pair: Term<RegPair>,
    registers: Term<Map<Reg8, Int>>,
    flags: Term<FlagSet>,
    sp: Term<Int>,
) -> Term<Int>;

k::equations! {
    for PairValue;
    pair_value_registers:
        (Pair16::Registers(high, low), registers, _, _)
        => { Pair(registers[high], registers[low]) };
    pair_value_af: (Pair16::Af, registers, flags, _) => Pair(registers[Register::A], FlagByte(flags));
    pair_value_sp: (Pair16::StackPointer, _, _, sp)  => sp;
}

/// Whether a branch condition holds for the flags.
#[k::function(total, internal)]
pub fn Holds(cond: Term<Cond>, flags: Term<FlagSet>) -> Term<Bool>;

k::equations! {
    for Holds;
    holds_always   : (Condition::Always, _)                 => true;
    holds_not_zero : (Condition::NotZero, Flags { z, .. })  => !z;
    holds_zero     : (Condition::Zero, Flags { z, .. })     => z;
    holds_not_carry: (Condition::NotCarry, Flags { c, .. }) => !c;
    holds_carry    : (Condition::Carry, Flags { c, .. })    => c;
}
