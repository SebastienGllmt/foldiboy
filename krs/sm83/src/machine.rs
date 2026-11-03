//! Architectural state, the bus, and the initial configuration.

use k::Term;
use k::builtin::{Bool, Int, KSequence, List, Map};

use crate::alu::{CarrySet, ZeroSet};
use crate::arith::Pair;
use crate::decode::{Cond, Condition, Pair16, RegPair};
use crate::interrupt::Step;

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
    /// The flag register: Z, N, H, C in bits 7 to 4; bits 3 to 0 are zero.
    #[k::cell]
    pub f: Int,
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
    /// vector gives the same result (`vectors --wz`).
    #[k::cell(unobservable)]
    pub w: Int,
    /// Internal temporary register, the low byte of `WZ`. Unobservable for the
    /// same reason as `w`, tested the same way.
    #[k::cell(unobservable)]
    pub z: Int,
    /// The address space, as a flat map from address to byte.
    #[k::cell]
    pub memory: Map<Int, Int>,
    /// The event log: what the CPU showed outside itself since the start, one
    /// event per machine cycle (the observation contract's vocabulary, `sm83-events`).
    #[k::cell]
    pub bus: List<CpuEvent>,
}

/// The initial state: an architectural state at an instruction boundary.
///
/// Every architectural cell is an input, so any boundary state can start a
/// run. The unobservable registers `W` and `Z` are inputs too, so a run can
/// test that their initial value changes no observation; the bus log starts
/// empty.
impl Machine {
    #[k::init]
    pub fn init(
        #[k::config_var(name = "REGS")] regs: Map<Reg8, Int>,
        #[k::config_var(name = "F")] f: Int,
        #[k::config_var(name = "PC")] pc: Int,
        #[k::config_var(name = "SP")] sp: Int,
        #[k::config_var(name = "IME")] ime: Bool,
        #[k::config_var(name = "EI_PENDING")] ei_pending: Bool,
        #[k::config_var(name = "HALTED")] halted: Bool,
        #[k::config_var(name = "HALT_BUG")] halt_bug: Bool,
        #[k::config_var(name = "MEMORY")] memory: Map<Int, Int>,
        #[k::config_var(name = "W")] w: Int,
        #[k::config_var(name = "Z")] z: Int,
    ) -> Self {
        Self {
            k: k::k![Step::Boundary],
            regs,
            f,
            pc,
            sp,
            ime,
            ei_pending,
            halted,
            halt_bug,
            w,
            z,
            memory,
            bus: k::builtin::list::empty(),
        }
    }
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
    flags: Term<Int>,
    sp: Term<Int>,
) -> Term<Int>;

k::equations! {
    for PairValue;
    pair_value_registers:
        (Pair16::Registers(high, low), registers, _, _)
        => { Pair(registers[high], registers[low]) };
    pair_value_af: (Pair16::Af, registers, flags, _) => Pair(registers[Register::A], flags);
    pair_value_sp: (Pair16::StackPointer, _, _, sp)  => sp;
}

/// Whether a branch condition holds for a flag byte.
#[k::function(total, internal)]
pub fn Holds(cond: Term<Cond>, flags: Term<Int>) -> Term<Bool>;

k::equations! {
    for Holds;
    holds_always   : (Condition::Always, _)       => true;
    holds_not_zero : (Condition::NotZero, flags)  => !ZeroSet(flags);
    holds_zero     : (Condition::Zero, flags)     => ZeroSet(flags);
    holds_not_carry: (Condition::NotCarry, flags) => !CarrySet(flags);
    holds_carry    : (Condition::Carry, flags)    => CarrySet(flags);
}
