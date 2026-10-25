//! The instruction boundary: the `EI` delay, `HALT` mode, and interrupt
//! dispatch.
//!
//! `IE` (`$FFFF`) and `IF` (`$FF0F`) are bytes of `memory`. The CPU reads
//! them over internal wires, not the bus, so these reads add nothing to the
//! bus log. An interrupt is requested when a bit 0 to 4 is set in both; the
//! lowest such bit is the one served.

use k::Term;
use k::builtin::{Bool, Int, Map, map};

use crate::Machine;
use crate::arith::{High, Low, Word};
use crate::machine::Cycle;
use crate::micro::{Micro, MicroOp};

#[k::constructors(sort = MicroOp, result = k::KResult)]
pub enum Step {
    /// Run `steps` boundary steps.
    #[k::syntax("#run({steps})")]
    Run { steps: Int },
    /// Apply the `EI` delay, then dispatch an interrupt, fetch the next
    /// instruction, or spend one cycle in `HALT` mode.
    #[k::syntax("#boundary")]
    Boundary,
    /// Push `PC` and jump to the vector of the requested interrupt.
    #[k::syntax("#dispatch")]
    Dispatch,
    /// Read the opcode at `PC` without advancing `PC` or executing it.
    #[k::syntax("#discardFetch")]
    DiscardFetch,
    /// Sample `IE`, then push the low byte of `PC` at `addr`.
    #[k::syntax("#pushLow({addr}, {value})")]
    PushLow { addr: Int, value: Int },
    /// Choose the vector from `ie` and the current `IF`, and acknowledge it.
    #[k::syntax("#selectVector({ie})")]
    SelectVector { ie: Int },
}

/// The interrupt enable register.
#[k::constant]
pub const IE: Int = 0xFFFF;

/// The interrupt flag register: the requested interrupts.
#[k::constant]
pub const IF: Int = 0xFF0F;

/// The interrupts both enabled and requested, as a bit set.
#[k::function(total, internal)]
pub fn Requested(ie: Term<Int>, flags: Term<Int>) -> Term<Int> {
    ie & flags & 0x1F
}

/// Whether `memory` holds an enabled and requested interrupt.
#[k::function(total, internal)]
pub fn InterruptRequested(memory: Term<Map<Int, Int>>) -> Term<Bool> {
    Requested(map::get(memory, IE), map::get(memory, IF)) != 0
}

/// Whether the boundary dispatches an interrupt.
#[k::function(total, internal)]
pub fn Dispatches(ime: Term<Bool>, memory: Term<Map<Int, Int>>) -> Term<Bool> {
    ime && InterruptRequested(memory)
}

/// The lowest bit of a request set, or 0 for the empty set.
#[k::function(total, internal)]
pub fn LowestRequest(requested: Term<Int>) -> Term<Int> {
    if requested & 0x01 != 0 {
        0x01
    } else if requested & 0x02 != 0 {
        0x02
    } else if requested & 0x04 != 0 {
        0x04
    } else if requested & 0x08 != 0 {
        0x08
    } else if requested & 0x10 != 0 {
        0x10
    } else {
        0
    }
}

/// The handler address of the interrupt served for a request set.
///
/// An empty set, which a push onto `IE` can cause, jumps to `$0000`.
#[k::function(total, internal)]
pub fn InterruptVector(requested: Term<Int>) -> Term<Int> {
    if requested & 0x01 != 0 {
        0x40 // VBlank
    } else if requested & 0x02 != 0 {
        0x48 // STAT
    } else if requested & 0x04 != 0 {
        0x50 // timer
    } else if requested & 0x08 != 0 {
        0x58 // serial
    } else if requested & 0x10 != 0 {
        0x60 // joypad
    } else {
        0x0000
    }
}

k::rules! {
    for Machine;

    #[k::requires(steps > 0)]
    run_step: {
        k: [Step::Run { steps }, ..],
    } => {
        k: [Step::Boundary, Step::Run { steps: steps - 1 }, ..],
    };

    #[k::requires(steps <= 0)]
    run_done: {
        k: [Step::Run { steps }, ..],
    } => {
        k: [..],
    };

    /// The check uses `IME` from before the `EI` delay is applied: `EI` sets
    /// `eiPending`, the boundary after `EI` still sees `IME` clear, and the
    /// boundary after the following instruction can dispatch.
    #[k::requires(!halted && Dispatches(ime, memory))]
    boundary_dispatch: {
        k: [Step::Boundary, ..],
        halted,
        ime,
        ei_pending: _,
        memory,
    } => {
        k: [Step::Dispatch, ..],
        ei_pending: false,
    };

    #[k::requires(!halted && !Dispatches(ime, memory))]
    boundary_fetch: {
        k: [Step::Boundary, ..],
        halted,
        ime,
        ei_pending: pending,
        memory,
    } => {
        k: [Micro::Fetch, ..],
        ime: ime || pending,
        ei_pending: false,
    };

    /// A requested interrupt ends `HALT` mode whether or not `IME` is set;
    /// the boundary then dispatches or fetches as usual.
    #[k::requires(halted && InterruptRequested(memory))]
    boundary_wake: {
        k: [Step::Boundary, ..],
        halted,
        memory,
    } => {
        k: [Step::Boundary, ..],
        halted: false,
    };

    #[k::requires(halted && !InterruptRequested(memory))]
    boundary_halted: {
        k: [Step::Boundary, ..],
        halted,
        memory,
    } => {
        k: [Micro::Idle, ..],
    };

    /// Five machine cycles: a discarded opcode fetch, two internal cycles,
    /// and the pushes of `PC`, high byte first.
    dispatch: {
        k: [Step::Dispatch, ..],
        pc,
        sp,
    } => {
        k: [
            Step::DiscardFetch,
            Micro::Idle,
            Micro::Idle,
            Micro::WriteByte { addr: Word(sp - 1), value: High(pc) },
            Step::PushLow { addr: Word(sp - 2), value: Low(pc) },
            ..
        ],
        sp: Word(sp - 2),
    };

    discard_fetch: {
        k: [Step::DiscardFetch, ..],
        pc,
        memory: { pc -> opcode, .. },
        bus: log,
    } => {
        k: [..],
        bus: [..log, Cycle::Read { addr: pc, value: opcode }],
    };

    /// `IE` is sampled after the high-byte push, so a push of the high byte
    /// onto `IE` (at `SP = $0000`) changes which interrupt is served, and a
    /// push of the low byte onto `IE` comes too late to.
    push_low: {
        k: [Step::PushLow { addr, value }, ..],
        memory: { IE -> ie, .. },
    } => {
        k: [Micro::WriteByte { addr, value }, Step::SelectVector { ie }, ..],
    };

    /// `IF` is sampled after both pushes; the served request is cleared and
    /// `IME` is cleared even when no request remains.
    select_vector: {
        k: [Step::SelectVector { ie }, ..],
        pc: _,
        ime: _,
        memory,
    } => {
        k: [..],
        pc: InterruptVector(Requested(ie, map::get(memory, IF))),
        ime: false,
        memory: map::update(
            memory,
            IF,
            map::get(memory, IF) - LowestRequest(Requested(ie, map::get(memory, IF))),
        ),
    };
}
