//! Architectural state, the bus, and the initial configuration.

use k::Term;
use k::builtin::{Bool, Bytes, Int, KSequence, List, Map, bytes};

use crate::alu::{FlagByte, FlagSet, Flags};
use crate::arith::Pair;
use crate::decode::{Cond, Condition, Pair16, RegPair};
use crate::interrupt::Step;

pub use sm83_events::{CpuEvent, Event};

k::sorts! {
    Reg8;
    Start;
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
}

/// Where a run starts: an architectural state at an instruction boundary,
/// every cell but the bus log, which starts empty.
///
/// A cartridge starts from the state its model's boot ROM leaves (`Boot`,
/// boot.rs). Any other boundary state can start a run too (SingleStepTests),
/// and so can any `W` and `Z`, so a run can test that their initial value
/// changes no observation.
#[k::constructor(sort = Start, syntax = "state({regs}, {f}, {pc}, {sp}, {ime}, {ei_pending}, {halted}, {halt_bug}, {rom}, {memory}, {w}, {z})")]
pub struct State {
    pub regs: Term<Map<Reg8, Int>>,
    pub f: Term<FlagSet>,
    pub pc: Term<Int>,
    pub sp: Term<Int>,
    pub ime: Term<Bool>,
    pub ei_pending: Term<Bool>,
    pub halted: Term<Bool>,
    pub halt_bug: Term<Bool>,
    pub rom: Term<Bytes>,
    pub memory: Term<Map<Int, Int>>,
    pub w: Term<Int>,
    pub z: Term<Int>,
}

/// The initial state: the bus log empty, every other cell as `start` states it.
impl Machine {
    #[k::init]
    pub fn init(#[k::config_var(name = "START")] start: Start) -> Self {
        Self {
            k: k::k![Step::Boundary],
            regs: self::StartRegs(start),
            f: self::StartF(start),
            pc: self::StartPc(start),
            sp: self::StartSp(start),
            ime: self::StartIme(start),
            ei_pending: self::StartEiPending(start),
            halted: self::StartHalted(start),
            halt_bug: self::StartHaltBug(start),
            memory: self::StartMemory(start),
            w: self::StartW(start),
            z: self::StartZ(start),
            rom_end: bytes::length(self::StartRom(start)),
            rom: self::StartRom(start),
            bus: k::builtin::list::empty(),
        }
    }
}

#[k::function(total, internal)]
pub fn StartRegs(start: Term<Start>) -> Term<Map<Reg8, Int>>;

k::equations! {
    for StartRegs;
    start_regs: State { regs, .. } => regs;
}

#[k::function(total, internal)]
pub fn StartF(start: Term<Start>) -> Term<FlagSet>;

k::equations! {
    for StartF;
    start_f: State { f, .. } => f;
}

#[k::function(total, internal)]
pub fn StartPc(start: Term<Start>) -> Term<Int>;

k::equations! {
    for StartPc;
    start_pc: State { pc, .. } => pc;
}

#[k::function(total, internal)]
pub fn StartSp(start: Term<Start>) -> Term<Int>;

k::equations! {
    for StartSp;
    start_sp: State { sp, .. } => sp;
}

#[k::function(total, internal)]
pub fn StartIme(start: Term<Start>) -> Term<Bool>;

k::equations! {
    for StartIme;
    start_ime: State { ime, .. } => ime;
}

#[k::function(total, internal)]
pub fn StartEiPending(start: Term<Start>) -> Term<Bool>;

k::equations! {
    for StartEiPending;
    start_ei_pending: State { ei_pending, .. } => ei_pending;
}

#[k::function(total, internal)]
pub fn StartHalted(start: Term<Start>) -> Term<Bool>;

k::equations! {
    for StartHalted;
    start_halted: State { halted, .. } => halted;
}

#[k::function(total, internal)]
pub fn StartHaltBug(start: Term<Start>) -> Term<Bool>;

k::equations! {
    for StartHaltBug;
    start_halt_bug: State { halt_bug, .. } => halt_bug;
}

#[k::function(total, internal)]
pub fn StartRom(start: Term<Start>) -> Term<Bytes>;

k::equations! {
    for StartRom;
    start_rom: State { rom, .. } => rom;
}

#[k::function(total, internal)]
pub fn StartMemory(start: Term<Start>) -> Term<Map<Int, Int>>;

k::equations! {
    for StartMemory;
    start_memory: State { memory, .. } => memory;
}

#[k::function(total, internal)]
pub fn StartW(start: Term<Start>) -> Term<Int>;

k::equations! {
    for StartW;
    start_w: State { w, .. } => w;
}

#[k::function(total, internal)]
pub fn StartZ(start: Term<Start>) -> Term<Int>;

k::equations! {
    for StartZ;
    start_z: State { z, .. } => z;
}

/// Whether `addr` is in the ROM region, 0 up to `rom_end`.
#[k::function(total, internal)]
pub fn InRom(rom_end: Term<Int>, addr: Term<Int>) -> Term<Bool> {
    0 <= addr && addr < rom_end
}

/// The value of the register pair `HL`.
#[k::function(total, internal)]
pub fn Hl(registers: Term<Map<Reg8, Int>>) -> Term<Int> {
    Pair(registers[Register::H], registers[Register::L])
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
