//! The SM83 observation contract: what two runs of the machine are compared on.
//!
//! An observation is the triple (end class, event log, end state)
//! (draft-personal/observer-design/design.md, D1). Every implementation of
//! the machine reports its run as one: a KRS definition through a runner that
//! reads its cells by name, any other system in the typed input encoding. No
//! semantics imports this crate, so its functions never take part in a
//! definition's critical-pair check and kink never mutates them.
//!
//! Two observations are compared after a tier (D5) and the named
//! normalizations (D6), applied to both sides. Every function here is total
//! and written as a left fold with an accumulator, so a statement about one
//! more event follows from `List.foldl_append` in Lean.

use k::Term;
use k::builtin::{Bool, Int, List, Map, kequal};
use sm83_events::{CpuEvent, Event};

k::sorts! {
    EndClass;
    EndState;
    Observation;
    Logged;
    Tier;
    Normalization;
    StateField;
    Verdict;
}

/// How a run ended.
#[k::constructors(sort = EndClass, result = k::KResult)]
pub enum End {
    /// The case's defined end: the CPU locked on an unused opcode.
    #[k::syntax("locked")]
    Locked,
    /// The run stopped on behavior the implementation does not support.
    #[k::syntax("stuck")]
    Stuck,
    /// The run was cut by a budget before it ended.
    #[k::syntax("bound")]
    Bound,
}

/// The architectural state at the end of a run: every cell a program can
/// observe, independent of how an implementation represents it.
#[k::constructors(sort = EndState, result = k::KResult)]
pub enum Architecture {
    #[k::syntax(
        "endState({a}, {b}, {c}, {d}, {e}, {h}, {l}, {zf}, {nf}, {hf}, {cf}, {pc}, {sp}, {ime}, {ei_pending}, {halted}, {halt_bug}, {memory})"
    )]
    State {
        a: Int,
        b: Int,
        c: Int,
        d: Int,
        e: Int,
        h: Int,
        l: Int,
        zf: Bool,
        nf: Bool,
        hf: Bool,
        cf: Bool,
        pc: Int,
        sp: Int,
        ime: Bool,
        ei_pending: Bool,
        halted: Bool,
        halt_bug: Bool,
        memory: Map<Int, Int>,
    },
}

/// One run's observation.
#[k::constructors(sort = Observation, result = k::KResult)]
pub enum Observed {
    #[k::syntax("observation({end}, {log}, {state})")]
    Run { end: EndClass, log: List<CpuEvent>, state: EndState },
}

/// An entry of a log under comparison: an event, or a run of idle cycles that
/// a normalization summarized.
#[k::constructors(sort = Logged, result = k::KResult)]
pub enum Entry {
    #[k::syntax("logged({event})")]
    Event { event: CpuEvent },
    #[k::syntax("idles({count})")]
    Idles { count: Int },
}

/// A named reduction of an observation to what a class of system can report.
#[k::constructors(sort = Tier, result = k::KResult)]
pub enum Reduction {
    /// The whole observation.
    #[k::syntax("full")]
    Full,
    /// The end class, the write events and the end state: no timing.
    #[k::syntax("writes")]
    Writes,
    /// The end class, the read and write events in order, and the end state: the bus
    /// transactions without their timing, for a system that does not model internal cycles.
    #[k::syntax("accesses")]
    Accesses,
}

/// A named, versioned normalization of event logs.
#[k::constructors(sort = Normalization, result = k::KResult)]
pub enum Normalize {
    /// Lossless: each run of consecutive idle cycles becomes one `idles(n)`.
    #[k::syntax("compressIdle")]
    CompressIdle,
}

/// A field of the end state, as a difference names it.
#[k::constructors(sort = StateField, result = k::KResult)]
pub enum Field {
    #[k::syntax("fieldA")]
    A,
    #[k::syntax("fieldB")]
    B,
    #[k::syntax("fieldC")]
    C,
    #[k::syntax("fieldD")]
    D,
    #[k::syntax("fieldE")]
    E,
    #[k::syntax("fieldH")]
    H,
    #[k::syntax("fieldL")]
    L,
    #[k::syntax("fieldZf")]
    Zf,
    #[k::syntax("fieldNf")]
    Nf,
    #[k::syntax("fieldHf")]
    Hf,
    #[k::syntax("fieldCf")]
    Cf,
    #[k::syntax("fieldPc")]
    Pc,
    #[k::syntax("fieldSp")]
    Sp,
    #[k::syntax("fieldIme")]
    Ime,
    #[k::syntax("fieldEiPending")]
    EiPending,
    #[k::syntax("fieldHalted")]
    Halted,
    #[k::syntax("fieldHaltBug")]
    HaltBug,
    #[k::syntax("fieldMemory")]
    Memory,
}

/// The result of a comparison: equal, or the first difference.
#[k::constructors(sort = Verdict, result = k::KResult)]
pub enum Compared {
    #[k::syntax("equal")]
    Equal,
    /// The runs ended differently.
    #[k::syntax("endDiffers({left}, {right})")]
    EndDiffers { left: EndClass, right: EndClass },
    /// The reduced logs first differ at `position`; `left` and `right` are
    /// each side's log from there (empty where that side's log ended).
    #[k::syntax("logDiffers({position}, {left}, {right})")]
    LogDiffers { position: Int, left: List<Logged>, right: List<Logged> },
    /// The end states first differ at `field`, in declaration order.
    #[k::syntax("stateDiffers({field})")]
    StateDiffers { field: StateField },
}

// --- Logs ---------------------------------------------------------------------------------

/// An event log as entries, unchanged.
#[k::function(total, internal)]
pub fn Lift(log: Term<List<CpuEvent>>) -> Term<List<Logged>> {
    LiftAcc(log, [])
}

#[k::function(total, internal)]
pub fn LiftAcc(log: Term<List<CpuEvent>>, acc: Term<List<Logged>>) -> Term<List<Logged>>;

k::equations! {
    for LiftAcc;
    lift_next: ([e, rest @ ..], acc) => LiftAcc(rest, [..acc, Entry::Event { event: e }]);
    /// The log is empty: a builtin list is empty or has a first element.
    #[k::equation(owise)]
    lift_end: (_, acc) => acc;
}

/// `CompressIdle`: each maximal run of idle cycles, written as idle events or
/// as earlier `idles(n)` entries, becomes one `idles(n)`. Idempotent, and
/// lossless: `Expand` restores the lifted log.
#[k::function(total, internal)]
pub fn Compress(log: Term<List<Logged>>) -> Term<List<Logged>> {
    CompressAcc(log, [], 0)
}

/// `pending` counts the idle cycles seen since the last other entry.
#[k::function(total, internal)]
pub fn CompressAcc(log: Term<List<Logged>>, acc: Term<List<Logged>>, pending: Term<Int>) -> Term<List<Logged>>;

k::equations! {
    for CompressAcc;
    /// The log is empty.
    #[k::equation(owise)]
    compress_end: (_, acc, n) => Flush(acc, n);
    compress_idle: ([Entry::Event { event: Event::Idle }, rest @ ..], acc, n) => CompressAcc(rest, acc, n + 1);
    compress_idles: ([Entry::Idles { count: m }, rest @ ..], acc, n) => CompressAcc(rest, acc, n + m);
    #[k::requires(n <= 0)]
    compress_read: ([x @ Entry::Event { event: Event::Read { .. } }, rest @ ..], acc, n)
        => CompressAcc(rest, [..acc, x], 0);
    #[k::requires(n > 0)]
    compress_read_run: ([x @ Entry::Event { event: Event::Read { .. } }, rest @ ..], acc, n)
        => CompressAcc(rest, [..acc, Entry::Idles { count: n }, x], 0);
    #[k::requires(n <= 0)]
    compress_write: ([x @ Entry::Event { event: Event::Write { .. } }, rest @ ..], acc, n)
        => CompressAcc(rest, [..acc, x], 0);
    #[k::requires(n > 0)]
    compress_write_run: ([x @ Entry::Event { event: Event::Write { .. } }, rest @ ..], acc, n)
        => CompressAcc(rest, [..acc, Entry::Idles { count: n }, x], 0);
}

/// `acc` with the pending idle cycles, if any, appended as one `idles(n)`.
#[k::function(total, internal)]
pub fn Flush(acc: Term<List<Logged>>, pending: Term<Int>) -> Term<List<Logged>>;

k::equations! {
    for Flush;
    #[k::requires(n <= 0)]
    flush_none: (acc, n) => acc;
    #[k::requires(n > 0)]
    flush_run: (acc, n) => [..acc, Entry::Idles { count: n }];
}

/// The inverse of `Compress` on its results: each `idles(n)` becomes `n` idle
/// events. Partial: a negative count has no expansion.
#[k::function(partial, internal)]
pub fn Expand(log: Term<List<Logged>>) -> Term<List<Logged>> {
    ExpandAcc(log, [])
}

#[k::function(partial, internal)]
pub fn ExpandAcc(log: Term<List<Logged>>, acc: Term<List<Logged>>) -> Term<List<Logged>>;

k::equations! {
    for ExpandAcc;
    expand_end: ([], acc) => acc;
    expand_event: ([x @ Entry::Event { .. }, rest @ ..], acc) => ExpandAcc(rest, [..acc, x]);
    expand_idles: ([Entry::Idles { count: n }, rest @ ..], acc)
        => ExpandAcc(rest, k::builtin::list::concat(acc, k::builtin::list::make(n, Entry::Event { event: Event::Idle })));
}

/// The named normalizations, applied in order.
#[k::function(total, internal)]
pub fn Normalized(log: Term<List<Logged>>, normalizations: Term<List<Normalization>>) -> Term<List<Logged>>;

k::equations! {
    for Normalized;
    normalized_compress_idle: (log, [Normalize::CompressIdle, rest @ ..]) => Normalized(Compress(log), rest);
    /// No normalization is left.
    #[k::equation(owise)]
    normalized_end: (log, _) => log;
}

/// The log a tier keeps.
#[k::function(total, internal)]
pub fn TierLog(tier: Term<Tier>, log: Term<List<Logged>>) -> Term<List<Logged>>;

k::equations! {
    for TierLog;
    tier_full: (Reduction::Full, log) => log;
    tier_writes: (Reduction::Writes, log) => WritesAcc(log, []);
    tier_accesses: (Reduction::Accesses, log) => AccessesAcc(log, []);
}

/// The write events of a log.
#[k::function(total, internal)]
pub fn WritesAcc(log: Term<List<Logged>>, acc: Term<List<Logged>>) -> Term<List<Logged>>;

k::equations! {
    for WritesAcc;
    writes_keep: ([x @ Entry::Event { event: Event::Write { .. } }, rest @ ..], acc) => WritesAcc(rest, [..acc, x]);
    writes_drop_read: ([Entry::Event { event: Event::Read { .. } }, rest @ ..], acc) => WritesAcc(rest, acc);
    writes_drop_idle: ([Entry::Event { event: Event::Idle }, rest @ ..], acc) => WritesAcc(rest, acc);
    writes_drop_idles: ([Entry::Idles { .. }, rest @ ..], acc) => WritesAcc(rest, acc);
    /// The log is empty.
    #[k::equation(owise)]
    writes_end: (_, acc) => acc;
}

/// The read and write events of a log.
#[k::function(total, internal)]
pub fn AccessesAcc(log: Term<List<Logged>>, acc: Term<List<Logged>>) -> Term<List<Logged>>;

k::equations! {
    for AccessesAcc;
    accesses_keep_read: ([x @ Entry::Event { event: Event::Read { .. } }, rest @ ..], acc) => AccessesAcc(rest, [..acc, x]);
    accesses_keep_write: ([x @ Entry::Event { event: Event::Write { .. } }, rest @ ..], acc) => AccessesAcc(rest, [..acc, x]);
    accesses_drop_idle: ([Entry::Event { event: Event::Idle }, rest @ ..], acc) => AccessesAcc(rest, acc);
    accesses_drop_idles: ([Entry::Idles { .. }, rest @ ..], acc) => AccessesAcc(rest, acc);
    /// The log is empty.
    #[k::equation(owise)]
    accesses_end: (_, acc) => acc;
}

// --- Comparison ---------------------------------------------------------------------------

/// Two observations compared under `tier` after `normalizations`: the end
/// classes, then the reduced logs, then the end states.
#[k::function(total, internal)]
pub fn Compare(
    left: Term<Observation>,
    right: Term<Observation>,
    tier: Term<Tier>,
    normalizations: Term<List<Normalization>>,
) -> Term<Verdict>;

k::equations! {
    for Compare;
    compare: (
        Observed::Run { end: left_end, log: left_log, state: left_state },
        Observed::Run { end: right_end, log: right_log, state: right_state },
        tier,
        normalizations,
    ) => First(
        EndCompare(left_end, right_end),
        First(
            LogCompare(
                TierLog(tier, Normalized(Lift(left_log), normalizations)),
                TierLog(tier, Normalized(Lift(right_log), normalizations)),
                0,
            ),
            StateCompare(left_state, right_state),
        ),
    );
}

/// `first` unless it is `equal`, then `second`.
#[k::function(total, internal)]
pub fn First(first: Term<Verdict>, second: Term<Verdict>) -> Term<Verdict>;

k::equations! {
    for First;
    first_equal: (Compared::Equal, second) => second;
    first_end: (first @ Compared::EndDiffers { .. }, _) => first;
    first_log: (first @ Compared::LogDiffers { .. }, _) => first;
    first_state: (first @ Compared::StateDiffers { .. }, _) => first;
}

#[k::function(total, internal)]
pub fn EndCompare(left: Term<EndClass>, right: Term<EndClass>) -> Term<Verdict> {
    kequal::ite(kequal::eq(left, right), Compared::Equal, Compared::EndDiffers { left, right })
}

/// The reduced logs from `position` on.
#[k::function(total, internal)]
pub fn LogCompare(left: Term<List<Logged>>, right: Term<List<Logged>>, position: Term<Int>) -> Term<Verdict>;

k::equations! {
    for LogCompare;
    log_equal: ([], [], _) => Compared::Equal;
    #[k::requires(kequal::eq(x, y))]
    log_same: ([x, left @ ..], [y, right @ ..], position) => LogCompare(left, right, position + 1);
    /// The first entries differ, or exactly one log has ended.
    #[k::equation(owise)]
    log_differs: (left, right, position) => Compared::LogDiffers { position, left, right };
}

/// `rest` if `same`, else a difference at `field`.
#[k::function(total, internal)]
pub fn FieldCompare(same: Term<Bool>, field: Term<StateField>, rest: Term<Verdict>) -> Term<Verdict> {
    kequal::ite(same, rest, Compared::StateDiffers { field })
}

/// The first differing field of two end states, in declaration order.
#[k::function(total, internal)]
pub fn StateCompare(left: Term<EndState>, right: Term<EndState>) -> Term<Verdict>;

k::equations! {
    for StateCompare;
    state_compare: (
        Architecture::State {
            a: a1, b: b1, c: c1, d: d1, e: e1, h: h1, l: l1,
            zf: zf1, nf: nf1, hf: hf1, cf: cf1, pc: pc1, sp: sp1,
            ime: ime1, ei_pending: ei1, halted: halted1, halt_bug: bug1, memory: m1,
        },
        Architecture::State {
            a: a2, b: b2, c: c2, d: d2, e: e2, h: h2, l: l2,
            zf: zf2, nf: nf2, hf: hf2, cf: cf2, pc: pc2, sp: sp2,
            ime: ime2, ei_pending: ei2, halted: halted2, halt_bug: bug2, memory: m2,
        },
    ) => FieldCompare(a1 == a2, Field::A,
        FieldCompare(b1 == b2, Field::B,
        FieldCompare(c1 == c2, Field::C,
        FieldCompare(d1 == d2, Field::D,
        FieldCompare(e1 == e2, Field::E,
        FieldCompare(h1 == h2, Field::H,
        FieldCompare(l1 == l2, Field::L,
        FieldCompare(kequal::eq(zf1, zf2), Field::Zf,
        FieldCompare(kequal::eq(nf1, nf2), Field::Nf,
        FieldCompare(kequal::eq(hf1, hf2), Field::Hf,
        FieldCompare(kequal::eq(cf1, cf2), Field::Cf,
        FieldCompare(pc1 == pc2, Field::Pc,
        FieldCompare(sp1 == sp2, Field::Sp,
        FieldCompare(kequal::eq(ime1, ime2), Field::Ime,
        FieldCompare(kequal::eq(ei1, ei2), Field::EiPending,
        FieldCompare(kequal::eq(halted1, halted2), Field::Halted,
        FieldCompare(kequal::eq(bug1, bug2), Field::HaltBug,
        FieldCompare(kequal::eq(m1, m2), Field::Memory,
        Compared::Equal))))))))))))))))));
}
