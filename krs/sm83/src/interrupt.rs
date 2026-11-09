//! The instruction boundary: the `EI` delay, `HALT` mode, and interrupt
//! dispatch.
//!
//! The machine runs forever, like the hardware: every boundary step queues
//! the next `#boundary` after the work it starts, so a computation cell
//! headed by `#boundary` is a state between two steps. Where to stop is the
//! observer's choice (a claim's postcondition, a depth bound), not the
//! machine's.
//!
//! `IE` (`$FFFF`) and `IF` (`$FF0F`) are bytes of `memory`. The CPU reads
//! them over internal wires, not the bus, so these reads add nothing to the
//! bus log. An interrupt is requested when a bit 0 to 4 is set in both; the
//! lowest such bit is the one served.
//!
//! The rest of the console (the timer, the PPU, the serial port, the joypad) requests interrupts
//! by setting bits of `IF`. The CPU stage has none of it, so its requests are an input of the
//! machine (`REQUESTS`, the `requests` cell): `deliver_request` sets each request's bits when its
//! machine cycle comes, also while the CPU is halted or dispatching, where no program can.

use k::Term;
use k::builtin::{Bool, Int, Map, bytes, list};

use crate::Machine;
use crate::arith::{High, Low, Word};
use crate::machine::{Event, InRom};
use crate::micro::{Micro, MicroOp};

#[k::constructors(sort = MicroOp, result = k::KResult)]
pub enum Step {
    /// Apply the `EI` delay, then dispatch an interrupt, fetch the next
    /// instruction, or spend one cycle in `HALT` mode; then the next boundary.
    #[k::syntax("#boundary")]
    Boundary,
    /// The CPU is locked by an unused opcode: no step follows.
    #[k::syntax("#locked")]
    Locked,
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

k::sorts! {
    InterruptRequest;
}

/// An interrupt request from the rest of the console.
#[k::constructors(sort = InterruptRequest, result = k::KResult)]
pub enum Request {
    /// `bits` are set in `IF` once `cycle` machine cycles have run (the bus log holds `cycle`
    /// events): every step from then on sees them.
    #[k::syntax("request({cycle}, {bits})")]
    At { cycle: Int, bits: Int },
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
    Requested(memory[IE], memory[IF]) != 0
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

    /// The next request of the rest of the console reaches `IF` once its cycle has come, before
    /// any other step (priority 10 runs ahead of every other rule, at K's default 50), so every
    /// read of `IF`, boundary decision and dispatch from then on sees it. The requests are in
    /// cycle order.
    #[k::rule(priority = 10)]
    #[k::requires(cycle <= list::size(log))]
    deliver_request: {
        requests: [Request::At { cycle, bits }, rest @ ..],
        memory,
        bus: log,
    } => {
        requests: rest,
        memory: k::map![IF -> memory[IF] | bits, ..memory],
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
        k: [Step::Dispatch, Step::Boundary, ..],
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
        k: [Micro::Fetch, Step::Boundary, ..],
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
        k: [Micro::Idle, Step::Boundary, ..],
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
            ..,
        ],
        sp: Word(sp - 2),
    };

    #[k::requires(!InRom(rom_end, pc))]
    discard_fetch: {
        k: [Step::DiscardFetch, ..],
        pc,
        rom_end,
        memory: { pc -> opcode, .. },
        bus: log,
    } => {
        k: [..],
        bus: [..log, Event::Read { addr: pc, value: opcode }],
    };

    /// As `discard_fetch`, reading the ROM.
    #[k::requires(InRom(rom_end, pc))]
    discard_fetch_rom: {
        k: [Step::DiscardFetch, ..],
        pc,
        rom,
        rom_end,
        bus: log,
    } => {
        k: [..],
        bus: [..log, Event::Read { addr: pc, value: bytes::get(rom, pc) }],
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
        pc: InterruptVector(Requested(ie, memory[IF])),
        ime: false,
        memory: k::map![IF -> memory[IF] - LowestRequest(Requested(ie, memory[IF])), ..memory],
    };
}
