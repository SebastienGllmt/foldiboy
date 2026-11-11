//! The situations the corpus must reach, as kink patterns over the SM83 definition's runs.
//!
//! Each clause is an obligation of the corpus: `krs kink query` finds a case whose run reaches it
//! and records that case as its witness in `corpus/kink.lock` (`[query]`), tries it first on the
//! next query and keeps it while it still hits, as `[run]` keeps a mutation's killer. A clause no
//! case reaches is recorded `open`, and `query --locked` fails when a recorded witness is lost.
//! So a new situation is written here first: if some case already reaches it, the lock names that
//! case and no case is added.
//!
//! A clause is a chain of steps, each a set of states written as a claim's subject: a cell a step
//! does not name is unconstrained. `a => b` holds when some later state of the same run matches
//! `b`, under the same binders; `unless(u)` on the arrow excludes a state matching `u` between
//! them. This package is no part of the definition: editing a clause re-asks that clause alone and
//! moves no verdict (corpus/README.md, "What each case tests").

use sm83::alu::{Flags, Unary};
use sm83::decode::Instruction;
use sm83::interrupt::{IF, INTERRUPT_BITS, Request, Step};
use sm83::machine::Event;
use sm83::micro::Micro;
use sm83::{InterruptRequested, Machine, Requested};

k::patterns! {
    for Machine;

    // --- HALT ---------------------------------------------------------------------------------

    /// The byte after a bugged HALT is read twice in a row: by the bugged fetch, then by the next
    /// fetch or as an operand. (Two adjacent reads of one address alone also match HALT with IME
    /// set and a request pending, whose dispatch re-reads the HALT's own address.)
    halt_bug_double_read: { k: [Micro::Fetch, ..], halt_bug: true, pc: p }
        => { bus: [.., Event::Read { addr: p, .. }, Event::Read { addr: p, .. }] };

    /// The bugged HALT's next opcode is `LD A, n`, which then reads its own opcode as its operand:
    /// the instruction the bugged fetch starts reads `$3E` at that fetch's address.
    halt_bug_on_two_byte_opcode: { k: [Micro::Fetch, ..], halt_bug: true, pc: p }
        => unless({ k: [Step::Boundary, ..] })
        { bus: [.., Event::Read { addr: p, value: 0x3E }] };

    /// The bugged HALT's next opcode is an `RST`, which then pushes its own address: the
    /// instruction the bugged fetch starts reads an `RST` opcode at that fetch's address.
    #[k::requires((opcode & 0xC7) == 0xC7)]
    halt_bug_then_rst: { k: [Micro::Fetch, ..], halt_bug: true, pc: p }
        => unless({ k: [Step::Boundary, ..] })
        { bus: [.., Event::Read { addr: p, value: opcode }] };

    /// HALT with IME clear and nothing requested enters HALT mode.
    halt_mode_ime_clear: { halted: true, ime: false };

    /// HALT with IME set and nothing requested enters HALT mode.
    halt_mode_ime_set: { halted: true, ime: true };

    /// In HALT mode with an enabled request pending: the wake condition.
    #[k::requires(InterruptRequested(memory))]
    halt_wake_request: { halted: true, memory };

    /// A request wakes HALT mode entered with IME clear: the CPU resumes fetching without
    /// dispatching, and the request stays in IF.
    #[k::requires(InterruptRequested(memory))]
    halt_wake_resumes: { halted: true, ime: false }
        => { k: [Micro::Fetch, ..], halted: false, memory };

    /// A request wakes HALT mode entered with IME set: the CPU dispatches it.
    halt_wake_dispatches: { halted: true, ime: true } => { k: [Step::Dispatch, ..] };

    // --- Interrupts ---------------------------------------------------------------------------

    /// `DI` runs with IME already set and an enabled request pending: `EI`'s delay kept the
    /// dispatch out.
    #[k::requires(InterruptRequested(memory))]
    ei_then_di: { k: [Instruction::DisableInterrupts, ..], ime: true, memory };

    /// The vector is selected with no enabled request left: the first push overwrote IE.
    #[k::requires(Requested(ie, flags) == 0)]
    dispatch_cancelled: { k: [Step::SelectVector { ie }, ..], memory: { IF -> flags, .. } };

    /// A dispatch starts with SP at `$0000`, so its first push writes IE.
    dispatch_from_sp_zero: { k: [Step::Dispatch, ..], sp: 0 };

    /// STAT (bit 1) and serial (bit 3) are both enabled and requested when a vector is selected.
    #[k::requires((Requested(ie, flags) & 0x0A) == 0x0A)]
    two_requests_at_selection: { k: [Step::SelectVector { ie }, ..], memory: { IF -> flags, .. } };

    /// A dispatch starts with only the timer requested, and VBlank is requested before the same
    /// dispatch selects its vector.
    #[k::requires((before & INTERRUPT_BITS) == 0x04)]
    #[k::requires((after & 0x05) == 0x05)]
    request_due_inside_dispatch: { k: [Step::Dispatch, ..], memory: { IF -> before, .. } }
        => unless({ k: [Step::Boundary, ..] })
        { k: [Step::SelectVector { .. }, ..], memory: { IF -> after, .. } };

    /// A read of IF sees a request made in its own machine cycle (VBlank) and not one made in the
    /// next (STAT, still to come); a later read of IF sees both.
    #[k::requires((seen & INTERRUPT_BITS) == 0x01)]
    #[k::requires((both & INTERRUPT_BITS) == 0x03)]
    if_read_sees_its_own_cycle: {
        bus: [.., Event::Read { addr: IF, value: seen }],
        requests: [Request::At { bits: 0x02, .. }, ..],
    } => { bus: [.., Event::Read { addr: IF, value: both }] };

    // --- Arithmetic ---------------------------------------------------------------------------

    /// DAA on a value above `$99` after an addition with C clear.
    #[k::requires(z > 0x99)]
    daa_above_99_no_carry_after_add: {
        k: [Micro::UpdateZ { op: Unary::Daa }, ..],
        z,
        f: Flags { n: false, c: false, .. },
    };

    /// DAA with a low digit above 9 after a subtraction with H clear: no correction applies.
    #[k::requires((z & 0x0F) > 9)]
    daa_low_digit_above_9_no_half_after_sub: {
        k: [Micro::UpdateZ { op: Unary::Daa }, ..],
        z,
        f: Flags { n: true, h: false, .. },
    };

    /// `ADD SP, e` or `LD HL, SP+e` with a negative offset.
    #[k::requires(z >= 0x80)]
    add_sp_negative_offset: { k: [Micro::AddSpZ { .. }, ..], z };

    // --- Memory -------------------------------------------------------------------------------

    /// A store to `$C110` is read back later with the same value.
    store_read_back: {
        bus: [.., Event::Write { addr: 0xC110, value: v }, .., Event::Read { addr: 0xC110, value: v }, ..],
    };
}
