//! The byte and word ranges claims state as premises.
//!
//! A claim states only the premises it needs, spelled with these predicates. Memory is a map
//! whose values cannot be quantified over in a premise, so a claim states `IsByte(m[addr])` for
//! each address it reads.

use k::Term;
use k::builtin::{Bool, Int};

/// A value an 8-bit register or memory cell holds.
#[k::function(total, internal)]
pub fn IsByte(x: Term<Int>) -> Term<Bool> {
    0 <= x && x < 0x100
}

/// A value `PC`, `SP` or an address holds.
#[k::function(total, internal)]
pub fn IsWord(x: Term<Int>) -> Term<Bool> {
    0 <= x && x < 0x1_0000
}
