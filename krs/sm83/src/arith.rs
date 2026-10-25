//! Fixed-width arithmetic on unbounded integers.

use k::Term;
use k::builtin::{Bool, Int, int};

/// The low 8 bits of an integer.
#[k::function(total, internal)]
pub fn Byte(value: Term<Int>) -> Term<Int> {
    int::rem_euclid(value, 0x100)
}

/// The low 16 bits of an integer.
#[k::function(total, internal)]
pub fn Word(value: Term<Int>) -> Term<Int> {
    int::rem_euclid(value, 0x1_0000)
}

/// The 16-bit value with high byte `high` and low byte `low`.
#[k::function(total, internal)]
pub fn Pair(high: Term<Int>, low: Term<Int>) -> Term<Int> {
    high * 0x100 + low
}

/// The high byte of a 16-bit value.
#[k::function(total, internal)]
pub fn High(value: Term<Int>) -> Term<Int> {
    value >> 8
}

/// The low byte of a 16-bit value.
#[k::function(total, internal)]
pub fn Low(value: Term<Int>) -> Term<Int> {
    value & 0xFF
}

/// A byte read as a two's-complement offset in `-128..=127`.
#[k::function(total, internal)]
pub fn Signed(value: Term<Int>) -> Term<Int> {
    if value < 0x80 { value } else { value - 0x100 }
}

/// `1` for `true`, `0` for `false`.
#[k::function(total, internal)]
pub fn FlagBit(flag: Term<Bool>) -> Term<Int>;

k::equations! {
    for FlagBit;
    flag_bit_set:   true  => 1;
    flag_bit_clear: false => 0;
}
