//! Arithmetic and logic: each operation's result byte and its flags.
//!
//! Results and flags are separate functions: a result depends only on the
//! operands and the carry, and flags are computed from the same inputs.

use k::Term;
use k::builtin::{Bool, Int};

use crate::arith::{Byte, FlagBit};

k::sorts! {
    AluOp;
    UnaryOp;
    FlagSet;
}

/// The two-operand operations on `A`, indexed by an opcode's `y` field.
#[k::constructors(sort = AluOp, result = k::KResult)]
pub enum Alu {
    #[k::syntax("add")]
    Add,
    #[k::syntax("adc")]
    Adc,
    #[k::syntax("sub")]
    Sub,
    #[k::syntax("sbc")]
    Sbc,
    #[k::syntax("and")]
    And,
    #[k::syntax("xor")]
    Xor,
    #[k::syntax("or")]
    Or,
    #[k::syntax("cp")]
    Cp,
}

/// The one-operand operations on a byte.
#[k::constructors(sort = UnaryOp, result = k::KResult)]
pub enum Unary {
    #[k::syntax("inc")]
    Inc,
    #[k::syntax("dec")]
    Dec,
    #[k::syntax("rlc")]
    Rlc,
    #[k::syntax("rrc")]
    Rrc,
    #[k::syntax("rl")]
    Rl,
    #[k::syntax("rr")]
    Rr,
    #[k::syntax("sla")]
    Sla,
    #[k::syntax("sra")]
    Sra,
    #[k::syntax("swap")]
    Swap,
    #[k::syntax("srl")]
    Srl,
    #[k::syntax("res {bit}")]
    ResetBit { bit: Int },
    #[k::syntax("set {bit}")]
    SetBit { bit: Int },
    /// The accumulator rotates: as `RLC`, `RRC`, `RL`, `RR`, but Z is cleared.
    #[k::syntax("rlca")]
    Rlca,
    #[k::syntax("rrca")]
    Rrca,
    #[k::syntax("rla")]
    Rla,
    #[k::syntax("rra")]
    Rra,
    /// Decimal adjust after a BCD addition or subtraction.
    #[k::syntax("daa")]
    Daa,
    /// Complement.
    #[k::syntax("cpl")]
    Cpl,
    /// Set the carry flag.
    #[k::syntax("scf")]
    Scf,
    /// Complement the carry flag.
    #[k::syntax("ccf")]
    Ccf,
}

/// The flag register: zero, subtract, half-carry and carry.
///
/// Only `PUSH AF` and `POP AF` see it as a byte (`FlagByte`, `ByteFlags`).
#[k::constructor(sort = FlagSet, syntax = "flags({z}, {n}, {h}, {c})")]
pub struct Flags {
    pub z: Term<Bool>,
    pub n: Term<Bool>,
    pub h: Term<Bool>,
    pub c: Term<Bool>,
}

/// The flag byte of `F`: Z, N, H, C in bits 7 to 4, bits 3 to 0 zero.
#[k::function(total, internal)]
pub fn FlagByte(flags: Term<FlagSet>) -> Term<Int> {
    match flags {
        Flags { z, n, h, c } => FlagBit(z) * 0x80 + FlagBit(n) * 0x40 + FlagBit(h) * 0x20 + FlagBit(c) * 0x10,
    }
}

/// The flags of a flag byte; bits 3 to 0 are ignored.
#[k::function(total, internal)]
pub fn ByteFlags(value: Term<Int>) -> Term<FlagSet> {
    Flags(value & 0x80 != 0, value & 0x40 != 0, value & 0x20 != 0, value & 0x10 != 0)
}

/// The carry flag, as `0` or `1`.
#[k::function(total, internal)]
pub fn CarryIn(flags: Term<FlagSet>) -> Term<Int> {
    FlagBit(CarrySet(flags))
}

/// Whether the zero flag is set.
#[k::function(total, internal)]
pub fn ZeroSet(flags: Term<FlagSet>) -> Term<Bool> {
    match flags {
        Flags { z, .. } => z,
    }
}

/// Whether the subtract flag is set.
#[k::function(total, internal)]
pub fn SubtractSet(flags: Term<FlagSet>) -> Term<Bool> {
    match flags {
        Flags { n, .. } => n,
    }
}

/// Whether the half-carry flag is set.
#[k::function(total, internal)]
pub fn HalfCarrySet(flags: Term<FlagSet>) -> Term<Bool> {
    match flags {
        Flags { h, .. } => h,
    }
}

/// Whether the carry flag is set.
#[k::function(total, internal)]
pub fn CarrySet(flags: Term<FlagSet>) -> Term<Bool> {
    match flags {
        Flags { c, .. } => c,
    }
}

/// The new `A` after `A <- A op b`.
#[k::function(total, internal)]
pub fn AluResult(op: Term<AluOp>, a: Term<Int>, b: Term<Int>, flags: Term<FlagSet>) -> Term<Int>;

/// The new flags after `A <- A op b`.
#[k::function(total, internal)]
pub fn AluFlags(op: Term<AluOp>, a: Term<Int>, b: Term<Int>, flags: Term<FlagSet>) -> Term<FlagSet>;

/// The new value after `v <- op v`.
#[k::function(total, internal)]
pub fn UnaryResult(op: Term<UnaryOp>, value: Term<Int>, flags: Term<FlagSet>) -> Term<Int>;

/// The new flags after `v <- op v`.
#[k::function(total, internal)]
pub fn UnaryFlags(op: Term<UnaryOp>, value: Term<Int>, flags: Term<FlagSet>) -> Term<FlagSet>;

k::equations! {
    for AluResult;
    add_result: (Alu::Add, a, b, _)     => Byte(a + b);
    adc_result: (Alu::Adc, a, b, flags) => Byte(a + b + CarryIn(flags));
    sub_result: (Alu::Sub, a, b, _)     => Byte(a - b);
    sbc_result: (Alu::Sbc, a, b, flags) => Byte(a - b - CarryIn(flags));
    and_result: (Alu::And, a, b, _)     => a & b;
    xor_result: (Alu::Xor, a, b, _)     => a ^ b;
    or_result:  (Alu::Or, a, b, _)      => a | b;
    /// `CP` compares by subtraction and discards the difference.
    cp_result:  (Alu::Cp, a, _, _)      => a;
}

k::equations! {
    for AluFlags;
    add_flags: (Alu::Add, a, b, _) => Flags(
        Byte(a + b) == 0,
        false,
        (a & 0x0F) + (b & 0x0F) > 0x0F,
        a + b > 0xFF,
    );
    adc_flags: (Alu::Adc, a, b, flags) => Flags(
        Byte(a + b + CarryIn(flags)) == 0,
        false,
        (a & 0x0F) + (b & 0x0F) + CarryIn(flags) > 0x0F,
        a + b + CarryIn(flags) > 0xFF,
    );
    sub_flags: (Alu::Sub, a, b, _) => Flags(
        a == b,
        true,
        a & 0x0F < b & 0x0F,
        a < b,
    );
    sbc_flags: (Alu::Sbc, a, b, flags) => Flags(
        Byte(a - b - CarryIn(flags)) == 0,
        true,
        a & 0x0F < (b & 0x0F) + CarryIn(flags),
        a < b + CarryIn(flags),
    );
    and_flags: (Alu::And, a, b, _)     => Flags(a & b == 0, false, true, false);
    xor_flags: (Alu::Xor, a, b, _)     => Flags(a == b, false, false, false);
    or_flags:  (Alu::Or, a, b, _)      => Flags(a | b == 0, false, false, false);
    cp_flags:  (Alu::Cp, a, b, flags)  => AluFlags(Alu::Sub, a, b, flags);
}

k::equations! {
    for UnaryResult;
    inc_result:       (Unary::Inc, value, _)          => Byte(value + 1);
    dec_result:       (Unary::Dec, value, _)          => Byte(value - 1);
    rlc_result:       (Unary::Rlc, value, _)          => Byte(value << 1) | value >> 7;
    rrc_result:       (Unary::Rrc, value, _)          => value >> 1 | (value & 0x01) << 7;
    rl_result:        (Unary::Rl, value, flags)       => Byte(value << 1) | CarryIn(flags);
    rr_result:        (Unary::Rr, value, flags)       => value >> 1 | CarryIn(flags) << 7;
    sla_result:       (Unary::Sla, value, _)          => Byte(value << 1);
    /// `SRA` keeps the sign bit.
    sra_result:       (Unary::Sra, value, _)          => value >> 1 | (value & 0x80);
    swap_result:      (Unary::Swap, value, _)         => (value & 0x0F) << 4 | value >> 4;
    srl_result:       (Unary::Srl, value, _)          => value >> 1;
    reset_bit_result: (Unary::ResetBit(bit), value, _) => value & (0xFF ^ 1 << bit);
    set_bit_result:   (Unary::SetBit(bit), value, _)  => value | 1 << bit;
    rlca_result:      (Unary::Rlca, value, flags)     => UnaryResult(Unary::Rlc, value, flags);
    rrca_result:      (Unary::Rrca, value, flags)     => UnaryResult(Unary::Rrc, value, flags);
    rla_result:       (Unary::Rla, value, flags)      => UnaryResult(Unary::Rl, value, flags);
    rra_result:       (Unary::Rra, value, flags)      => UnaryResult(Unary::Rr, value, flags);
    #[k::requires(!SubtractSet(flags))]
    daa_result_after_add: (Unary::Daa, value, flags)  => Byte(value + DaaCorrection(value, flags));
    #[k::requires(SubtractSet(flags))]
    daa_result_after_sub: (Unary::Daa, value, flags)  => Byte(value - DaaCorrection(value, flags));
    cpl_result:       (Unary::Cpl, value, _)          => value ^ 0xFF;
    scf_result:       (Unary::Scf, value, _)          => value;
    ccf_result:       (Unary::Ccf, value, _)          => value;
}

k::equations! {
    for UnaryFlags;
    /// `INC` and `DEC` leave the carry flag unchanged.
    inc_flags: (Unary::Inc, value, flags) => Flags(
        value == 0xFF,
        false,
        value & 0x0F == 0x0F,
        CarrySet(flags),
    );
    dec_flags: (Unary::Dec, value, flags) => Flags(
        value == 0x01,
        true,
        value & 0x0F == 0x00,
        CarrySet(flags),
    );
    rlc_flags:  (Unary::Rlc, value, flags) => ShiftFlags(UnaryResult(Unary::Rlc, value, flags), value & 0x80 != 0);
    rrc_flags:  (Unary::Rrc, value, flags) => ShiftFlags(UnaryResult(Unary::Rrc, value, flags), value & 0x01 != 0);
    rl_flags:   (Unary::Rl, value, flags)  => ShiftFlags(UnaryResult(Unary::Rl, value, flags), value & 0x80 != 0);
    rr_flags:   (Unary::Rr, value, flags)  => ShiftFlags(UnaryResult(Unary::Rr, value, flags), value & 0x01 != 0);
    sla_flags:  (Unary::Sla, value, flags) => ShiftFlags(UnaryResult(Unary::Sla, value, flags), value & 0x80 != 0);
    sra_flags:  (Unary::Sra, value, flags) => ShiftFlags(UnaryResult(Unary::Sra, value, flags), value & 0x01 != 0);
    srl_flags:  (Unary::Srl, value, flags) => ShiftFlags(UnaryResult(Unary::Srl, value, flags), value & 0x01 != 0);
    swap_flags: (Unary::Swap, value, _)    => Flags(value == 0, false, false, false);
    /// `RES` and `SET` leave the flags unchanged.
    bit_update_flags: (Unary::ResetBit(_) | Unary::SetBit(_), _, flags) => flags;
    rlca_flags: (Unary::Rlca, value, _)    => Flags(false, false, false, value & 0x80 != 0);
    rrca_flags: (Unary::Rrca, value, _)    => Flags(false, false, false, value & 0x01 != 0);
    rla_flags:  (Unary::Rla, value, _)     => Flags(false, false, false, value & 0x80 != 0);
    rra_flags:  (Unary::Rra, value, _)     => Flags(false, false, false, value & 0x01 != 0);
    daa_flags:  (Unary::Daa, value, flags) => Flags(
        UnaryResult(Unary::Daa, value, flags) == 0,
        SubtractSet(flags),
        false,
        DaaHigh(value, flags),
    );
    cpl_flags:  (Unary::Cpl, _, flags)     => Flags(ZeroSet(flags), true, true, CarrySet(flags));
    scf_flags:  (Unary::Scf, _, flags)     => Flags(ZeroSet(flags), false, false, true);
    ccf_flags:  (Unary::Ccf, _, flags)     => Flags(ZeroSet(flags), false, false, !CarrySet(flags));
}

/// The flags of a rotate or shift: Z from the result, C from the bit
/// shifted out.
#[k::function(total, internal)]
pub fn ShiftFlags(result: Term<Int>, carry: Term<Bool>) -> Term<FlagSet> {
    Flags(result == 0, false, false, carry)
}

/// The new flags after `BIT bit, value`; the value itself is unchanged.
#[k::function(total, internal)]
pub fn BitFlags(bit: Term<Int>, value: Term<Int>, flags: Term<FlagSet>) -> Term<FlagSet> {
    Flags(value & 1 << bit == 0, false, true, CarrySet(flags))
}

/// Whether `DAA` corrects the high digit, which is also its carry out.
///
/// After a subtraction a digit is corrected only if it borrowed; after an
/// addition, also if it exceeds 9.
#[k::function(total, internal)]
pub fn DaaHigh(value: Term<Int>, flags: Term<FlagSet>) -> Term<Bool> {
    if SubtractSet(flags) {
        CarrySet(flags)
    } else {
        CarrySet(flags) || value > 0x99
    }
}

/// Whether `DAA` corrects the low digit.
#[k::function(total, internal)]
pub fn DaaLow(value: Term<Int>, flags: Term<FlagSet>) -> Term<Bool> {
    if SubtractSet(flags) {
        HalfCarrySet(flags)
    } else {
        HalfCarrySet(flags) || value & 0x0F > 0x09
    }
}

/// The correction `DAA` adds after an addition or subtracts after a
/// subtraction.
#[k::function(total, internal)]
pub fn DaaCorrection(value: Term<Int>, flags: Term<FlagSet>) -> Term<Int> {
    FlagBit(DaaHigh(value, flags)) * 0x60 + FlagBit(DaaLow(value, flags)) * 0x06
}

/// The flags of `ADD HL, rr`: Z unchanged, carries out of bits 11 and 15.
#[k::function(total, internal)]
pub fn AddHlFlags(hl: Term<Int>, value: Term<Int>, flags: Term<FlagSet>) -> Term<FlagSet> {
    Flags(
        ZeroSet(flags),
        false,
        (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF,
        hl + value > 0xFFFF,
    )
}

/// The flags of `SP + e`: carries out of bits 3 and 7 of the low byte,
/// adding the offset byte unsigned.
#[k::function(total, internal)]
pub fn SpOffsetFlags(sp: Term<Int>, offset: Term<Int>) -> Term<FlagSet> {
    Flags(
        false,
        false,
        (sp & 0x0F) + (offset & 0x0F) > 0x0F,
        (sp & 0xFF) + offset > 0xFF,
    )
}
