//! The event vocabulary of Foldiboy's SM83 observation contract.
//!
//! An event is behavior visible outside the CPU during a run. A definition's
//! rules append one to its event log in the same rewrite step that performs
//! the action; rules that only move values between internal registers append
//! nothing. Every implementation compared under the contract (the KRS
//! reference, refinement layers, the Rust emulator, other emulators) reports
//! its run in these terms, so this crate holds the vocabulary and nothing else
//! (draft-personal/observer-design/design.md, D2 and D3).
//!
//! Events are ordered. The CPU stage logs one event per machine cycle, but the
//! vocabulary does not promise that: the console stage may add events that
//! coincide with a bus cycle (interrupt acknowledgement, DMA, input samples)
//! under a new contract version.

use k::builtin::Int;

// CpuEvent: an event of a run, the sort of an event-log cell.
k::sorts! {
    CpuEvent;
}

/// The constructors of [`CpuEvent`].
#[k::constructors(sort = CpuEvent, result = k::KResult)]
pub enum Event {
    /// The CPU read `value` from `addr`.
    #[k::syntax("read({addr}, {value})")]
    Read { addr: Int, value: Int },
    /// The CPU wrote `value` to `addr`. The byte it replaced is not part of the
    /// event: it is memory at `addr` before the write, which any consumer that
    /// needs it derives.
    #[k::syntax("write({addr}, {value})")]
    Write { addr: Int, value: Int },
    /// A machine cycle with no bus access.
    #[k::syntax("idle")]
    Idle,
}
