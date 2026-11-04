//! The well-formed states of the SM83 definition (draft-personal/corpus-contract D-WELLFORMED).
//!
//! Claims state their premises with these predicates instead of restating byte and word ranges,
//! and a refinement theorem quantifies over them. Memory is a map whose values cannot be
//! quantified over in a premise, so a claim states `IsByte(m[addr])` for each address it reads;
//! case tooling checks a concrete initial memory byte by byte.

use crate::machine::{Reg8, Register};
use k::builtin::{Bool, Int, Map};
use k::Term;

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

/// Every 8-bit register is present and holds a byte.
#[k::function(total, internal)]
pub fn WellFormedRegisters(r: Term<Map<Reg8, Int>>) -> Term<Bool> {
    IsByte(r[Register::A])
        && IsByte(r[Register::B])
        && IsByte(r[Register::C])
        && IsByte(r[Register::D])
        && IsByte(r[Register::E])
        && IsByte(r[Register::H])
        && IsByte(r[Register::L])
}

/// The register file, `PC` and `SP` are well formed.
/// The flags are four Booleans and the other control cells are Booleans, so they need no premise.
#[k::function(total, internal)]
pub fn WellFormed(r: Term<Map<Reg8, Int>>, pc: Term<Int>, sp: Term<Int>) -> Term<Bool> {
    WellFormedRegisters(r) && IsWord(pc) && IsWord(sp)
}
