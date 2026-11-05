//! Views of the SM83 definition's cells: functions of one cell's value that a comparison applies
//! to both sides, so two ends are compared on what a use observes of the cell
//! (`krs compare --view CELL=FUNCTION`, a case's `expect.views`).
//!
//! This package is an observer: it depends on the definition, and no semantics imports it, so
//! editing a view changes neither the definition, its coverage, nor anything keyed by it. Run a
//! comparison with `--observer foldiboy-sm83-views:views`.
//!
//! The bus log views are the observation contract's tiers (draft-personal/observer-design,
//! D5): the whole log (no view), the reads and writes in order (`AccessesView`, for a system
//! without internal cycles that orders its accesses as the hardware does), and the writes alone
//! (`WritesView`, for a system that orders its reads otherwise, such as the Rust emulator's CPU).
//! Each is a left fold with an accumulator, so a statement about one more event follows from
//! `List.foldl_append` in Lean.

use k::Term;
use k::builtin::{Int, List, Map};
use sm83::machine::{Reg8, Register};
use sm83_events::{CpuEvent, Event};

/// The write events of a bus log, in order.
#[k::function(total, internal)]
pub fn WritesView(log: Term<List<CpuEvent>>) -> Term<List<CpuEvent>> {
    Writes(log, [])
}

#[k::function(total, internal)]
pub fn Writes(log: Term<List<CpuEvent>>, acc: Term<List<CpuEvent>>) -> Term<List<CpuEvent>>;

k::equations! {
    for Writes;
    writes_keep: ([x @ Event::Write { .. }, rest @ ..], acc) => Writes(rest, [..acc, x]);
    writes_drop_read: ([Event::Read { .. }, rest @ ..], acc) => Writes(rest, acc);
    writes_drop_idle: ([Event::Idle, rest @ ..], acc) => Writes(rest, acc);
    /// The log is empty: a builtin list is empty or has a first element.
    #[k::equation(owise)]
    writes_end: (_, acc) => acc;
}

/// The read and write events of a bus log, in order: the log without its idle cycles.
#[k::function(total, internal)]
pub fn AccessesView(log: Term<List<CpuEvent>>) -> Term<List<CpuEvent>> {
    Accesses(log, [])
}

#[k::function(total, internal)]
pub fn Accesses(log: Term<List<CpuEvent>>, acc: Term<List<CpuEvent>>) -> Term<List<CpuEvent>>;

k::equations! {
    for Accesses;
    accesses_keep_read: ([x @ Event::Read { .. }, rest @ ..], acc) => Accesses(rest, [..acc, x]);
    accesses_keep_write: ([x @ Event::Write { .. }, rest @ ..], acc) => Accesses(rest, [..acc, x]);
    accesses_drop_idle: ([Event::Idle, rest @ ..], acc) => Accesses(rest, acc);
    /// The log is empty.
    #[k::equation(owise)]
    accesses_end: (_, acc) => acc;
}

/// The registers a mooneye test reports its result in: `B C D E H L` (3 5 8 13 21 34 is a pass,
/// all `$42` a failure). `A` is not part of the result.
#[k::function(total, internal)]
pub fn MooneyeResult(regs: Term<Map<Reg8, Int>>) -> Term<Map<Reg8, Int>> {
    k::map![
        Register::B -> regs[Register::B],
        Register::C -> regs[Register::C],
        Register::D -> regs[Register::D],
        Register::E -> regs[Register::E],
        Register::H -> regs[Register::H],
        Register::L -> regs[Register::L],
    ]
}
