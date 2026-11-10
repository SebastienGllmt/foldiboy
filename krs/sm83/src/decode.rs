//! Instruction decoding.
//!
//! An opcode is written as its bits `xx yyy zzz`, with `y` further split
//! into `pp q`; a regular instruction group is one bit template whose
//! fields index fixed operand tables (`cc` is the low two bits of `y`).

use k::Term;
use k::builtin::Int;

use crate::alu::{Alu, AluOp, Unary, UnaryOp};
use crate::machine::{Reg8, Register};

k::sorts! {
    Operand8;
    RegPair;
    Cond;
    AddrMode;
    Instr: k::KItem;
}

/// An 8-bit operand.
#[k::constructors(sort = Operand8, result = k::KResult)]
pub enum Operand {
    #[k::syntax("reg({r})")]
    Reg { r: Reg8 },
    /// The byte addressed by `HL`.
    #[k::syntax("(hl)")]
    HlIndirect,
    /// The byte following the opcode.
    #[k::syntax("n")]
    Immediate,
}

/// A 16-bit register pair.
#[k::constructors(sort = RegPair, result = k::KResult)]
pub enum Pair16 {
    /// `BC`, `DE`, or `HL`: two registers of the register file.
    #[k::syntax("pair({high}, {low})")]
    Registers { high: Reg8, low: Reg8 },
    #[k::syntax("af")]
    Af,
    #[k::syntax("sp")]
    StackPointer,
}

/// A branch condition on the flags.
#[k::constructors(sort = Cond, result = k::KResult)]
pub enum Condition {
    #[k::syntax("always")]
    Always,
    #[k::syntax("nz")]
    NotZero,
    #[k::syntax("z")]
    Zero,
    #[k::syntax("nc")]
    NotCarry,
    #[k::syntax("cy")]
    Carry,
}

/// How a load or store through `A` forms its address.
#[k::constructors(sort = AddrMode, result = k::KResult)]
pub enum Address {
    /// `(BC)` or `(DE)`.
    #[k::syntax("({high}{low})")]
    ViaPair { high: Reg8, low: Reg8 },
    /// `(HL+)` or `(HL-)`: `HL`, which then steps by `delta`.
    #[k::syntax("(hl step {delta})")]
    ViaHlStep { delta: Int },
    /// `(0xFF00 + n)`.
    #[k::syntax("(high n)")]
    HighImmediate,
    /// `(0xFF00 + C)`.
    #[k::syntax("(high c)")]
    HighC,
    /// `(nn)`.
    #[k::syntax("(nn)")]
    Absolute,
}

/// A decoded instruction.
#[k::constructors(sort = Instr, result = k::KResult)]
pub enum Instruction {
    #[k::syntax("nop")]
    Nop,
    /// `LD dst, src`.
    #[k::syntax("ld {dst}, {src}")]
    Ld8 { dst: Operand8, src: Operand8 },
    /// `LD A, (addr)`.
    #[k::syntax("ld a, {addr}")]
    LoadA { addr: AddrMode },
    /// `LD (addr), A`.
    #[k::syntax("ld {addr}, a")]
    StoreA { addr: AddrMode },
    /// `LD pair, nn`.
    #[k::syntax("ld {pair}, nn")]
    Ld16 { pair: RegPair },
    /// `LD (nn), SP`.
    #[k::syntax("ld (nn), sp")]
    StoreSp,
    /// `LD SP, HL`.
    #[k::syntax("ld sp, hl")]
    LdSpHl,
    /// `LD HL, SP + e`.
    #[k::syntax("ld hl, sp + e")]
    LdHlSpOffset,
    /// `ADD SP, e`.
    #[k::syntax("add sp, e")]
    AddSpOffset,
    #[k::syntax("inc {pair}")]
    Inc16 { pair: RegPair },
    #[k::syntax("dec {pair}")]
    Dec16 { pair: RegPair },
    /// `ADD HL, pair`.
    #[k::syntax("add hl, {pair}")]
    AddHl { pair: RegPair },
    /// `op A, src`.
    #[k::syntax("{op} a, {src}")]
    Alu8 { op: AluOp, src: Operand8 },
    /// `op target`: read, modify, and write back one operand.
    #[k::syntax("{op} {target}")]
    Update8 { op: UnaryOp, target: Operand8 },
    /// `BIT bit, target`.
    #[k::syntax("bit {bit}, {target}")]
    TestBit { bit: Int, target: Operand8 },
    /// `JR cond, e`.
    #[k::syntax("jr {cond}, e")]
    JumpRelative { cond: Cond },
    /// `JP cond, nn`.
    #[k::syntax("jp {cond}, nn")]
    Jump { cond: Cond },
    /// `JP HL`.
    #[k::syntax("jp hl")]
    JumpHl,
    /// `CALL cond, nn`.
    #[k::syntax("call {cond}, nn")]
    Call { cond: Cond },
    #[k::syntax("ret")]
    Return,
    /// `RET cond`: unlike `RET`, it spends a cycle testing the condition.
    #[k::syntax("ret {cond}")]
    ReturnIf { cond: Cond },
    #[k::syntax("reti")]
    ReturnFromInterrupt,
    /// `RST target`.
    #[k::syntax("rst {target}")]
    Restart { target: Int },
    #[k::syntax("push {pair}")]
    Push { pair: RegPair },
    #[k::syntax("pop {pair}")]
    Pop { pair: RegPair },
    #[k::syntax("di")]
    DisableInterrupts,
    #[k::syntax("ei")]
    EnableInterrupts,
    #[k::syntax("halt")]
    Halt,
    /// `0xCB`: the next byte selects a second-page instruction.
    #[k::syntax("prefix cb")]
    Prefix,
    /// `STOP`: not supported by this definition; no rule executes it.
    #[k::syntax("stop")]
    Stop,
    /// One of the eleven unused opcodes, which lock the CPU.
    #[k::syntax("#illegal({opcode})")]
    Illegal { opcode: Int },
}

/// The instruction an opcode encodes; every byte has one.
#[k::function(total, internal, underspecified = "an opcode is a byte")]
pub fn Decode(opcode: Term<Int>) -> Term<Instr>;

/// The instruction an opcode after the `0xCB` prefix encodes.
#[k::function(total, internal, underspecified = "an opcode is a byte")]
pub fn DecodeCb(opcode: Term<Int>) -> Term<Instr>;

/// The 8-bit operand table `r`.
#[k::function(total, internal, underspecified = "indexed by a 3-bit field")]
pub fn R8(index: Term<Int>) -> Term<Operand8>;

/// The register-pair table `rp`.
#[k::function(total, internal, underspecified = "indexed by a 2-bit field")]
pub fn Rp(index: Term<Int>) -> Term<RegPair>;

/// The register-pair table `rp2` of `PUSH` and `POP`.
#[k::function(total, internal, underspecified = "indexed by a 2-bit field")]
pub fn Rp2(index: Term<Int>) -> Term<RegPair>;

/// The addressing table of `LD (rp), A` and `LD A, (rp)`, indexed by `p`.
#[k::function(total, internal, underspecified = "indexed by a 2-bit field")]
pub fn IndirectTable(index: Term<Int>) -> Term<AddrMode>;

/// The accumulator and flag operations of `x = 0, z = 7`, indexed by `y`.
#[k::function(total, internal, underspecified = "indexed by a 3-bit field")]
pub fn AccumulatorTable(index: Term<Int>) -> Term<UnaryOp>;

/// The condition table `cc`.
#[k::function(total, internal, underspecified = "indexed by a 2-bit field")]
pub fn Cc(index: Term<Int>) -> Term<Cond>;

/// The arithmetic table `alu`.
#[k::function(total, internal, underspecified = "indexed by a 3-bit field")]
pub fn AluTable(index: Term<Int>) -> Term<AluOp>;

/// The rotation and shift table `rot`.
#[k::function(total, internal, underspecified = "indexed by a 3-bit field")]
pub fn Rot(index: Term<Int>) -> Term<UnaryOp>;

k::equations! {
    for Decode;
    use Instruction::*;
    decode_nop:              0x00                        => Nop;
    decode_ld_nn_sp:         0x08                        => StoreSp;
    decode_jr:               0x18                        => JumpRelative(Condition::Always);
    decode_jr_cc:            k::bits!("00 1cc 000")      => JumpRelative(Cc(c));
    decode_ld_rp_nn:         k::bits!("00 pp0 001")      => Ld16(Rp(p));
    decode_add_hl:           k::bits!("00 pp1 001")      => AddHl(Rp(p));
    /// `LD (BC), A`, `LD (DE), A`, `LD (HL+), A`, `LD (HL-), A`.
    decode_store_a_indirect: k::bits!("00 pp0 010")      => StoreA(IndirectTable(p));
    /// `LD A, (BC)`, `LD A, (DE)`, `LD A, (HL+)`, `LD A, (HL-)`.
    decode_load_a_indirect:  k::bits!("00 pp1 010")      => LoadA(IndirectTable(p));
    decode_inc_rp:           k::bits!("00 pp0 011")      => Inc16(Rp(p));
    decode_dec_rp:           k::bits!("00 pp1 011")      => Dec16(Rp(p));
    decode_inc_r:            k::bits!("00 yyy 100")      => Update8(Unary::Inc, R8(y));
    decode_dec_r:            k::bits!("00 yyy 101")      => Update8(Unary::Dec, R8(y));
    decode_ld_r_n:           k::bits!("00 yyy 110")      => Ld8(R8(y), Operand::Immediate);
    /// `RLCA`, `RRCA`, `RLA`, `RRA`, `DAA`, `CPL`, `SCF`, `CCF`.
    decode_accumulator:      k::bits!("00 yyy 111")
                             => Update8(AccumulatorTable(y), Operand::Reg(Register::A));
    /// `LD (HL), (HL)` is `HALT`.
    decode_halt:             k::bits!("01 110 110")      => Halt;
    #[k::requires(op != 0x76)]
    decode_ld_r_r:           op @ k::bits!("01 yyy zzz") => Ld8(R8(y), R8(z));
    decode_alu_r:            k::bits!("10 yyy zzz")      => Alu8(AluTable(y), R8(z));
    decode_ret_cc:           k::bits!("11 0cc 000")      => ReturnIf(Cc(c));
    decode_ldh_n_a:          0xE0                        => StoreA(Address::HighImmediate);
    decode_add_sp_e:         0xE8                        => AddSpOffset;
    decode_ldh_a_n:          0xF0                        => LoadA(Address::HighImmediate);
    decode_ld_hl_sp_e:       0xF8                        => LdHlSpOffset;
    decode_pop:              k::bits!("11 pp0 001")      => Pop(Rp2(p));
    decode_ret:              0xC9                        => Return;
    decode_reti:             0xD9                        => ReturnFromInterrupt;
    decode_jp_hl:            0xE9                        => JumpHl;
    decode_ld_sp_hl:         0xF9                        => LdSpHl;
    decode_jp_cc:            k::bits!("11 0cc 010")      => Jump(Cc(c));
    decode_ld_c_a:           0xE2                        => StoreA(Address::HighC);
    decode_ld_nn_a:          0xEA                        => StoreA(Address::Absolute);
    decode_ld_a_c:           0xF2                        => LoadA(Address::HighC);
    decode_ld_a_nn:          0xFA                        => LoadA(Address::Absolute);
    decode_jp:               0xC3                        => Jump(Condition::Always);
    decode_prefix:           0xCB                        => Prefix;
    decode_di:               0xF3                        => DisableInterrupts;
    decode_ei:               0xFB                        => EnableInterrupts;
    decode_call_cc:          k::bits!("11 0cc 100")      => Call(Cc(c));
    decode_push:             k::bits!("11 pp0 101")      => Push(Rp2(p));
    decode_call:             0xCD                        => Call(Condition::Always);
    decode_alu_n:            k::bits!("11 yyy 110")      => Alu8(AluTable(y), Operand::Immediate);
    decode_rst:              k::bits!("11 yyy 111")      => Restart(y * 8);
    decode_stop:             0x10                        => Stop;
    /// The unused opcodes.
    decode_illegal:          opcode @ (0xD3
                             | 0xDB
                             | 0xDD
                             | 0xE3
                             | 0xE4
                             | 0xEB
                             | 0xEC
                             | 0xED
                             | 0xF4
                             | 0xFC
                             | 0xFD) => Illegal(opcode);
}

k::equations! {
    for DecodeCb;
    use Instruction::*;
    decode_cb_rot: k::bits!("00 yyy zzz") => Update8(Rot(y), R8(z));
    decode_cb_bit: k::bits!("01 yyy zzz") => TestBit(y, R8(z));
    decode_cb_res: k::bits!("10 yyy zzz") => Update8(Unary::ResetBit(y), R8(z));
    decode_cb_set: k::bits!("11 yyy zzz") => Update8(Unary::SetBit(y), R8(z));
}

k::equations! {
    for R8;
    use Operand::*;
    use Register::*;
    r8_b:           0 => Reg(B);
    r8_c:           1 => Reg(C);
    r8_d:           2 => Reg(D);
    r8_e:           3 => Reg(E);
    r8_h:           4 => Reg(H);
    r8_l:           5 => Reg(L);
    r8_hl_indirect: 6 => HlIndirect;
    r8_a:           7 => Reg(A);
}

k::equations! {
    for Rp;
    use Register::*;
    rp_bc: 0 => Pair16::Registers(B, C);
    rp_de: 1 => Pair16::Registers(D, E);
    rp_hl: 2 => Pair16::Registers(H, L);
    rp_sp: 3 => Pair16::StackPointer;
}

k::equations! {
    for Rp2;
    use Register::*;
    rp2_bc: 0 => Pair16::Registers(B, C);
    rp2_de: 1 => Pair16::Registers(D, E);
    rp2_hl: 2 => Pair16::Registers(H, L);
    rp2_af: 3 => Pair16::Af;
}

k::equations! {
    for IndirectTable;
    use Register::*;
    indirect_bc:           0 => Address::ViaPair(B, C);
    indirect_de:           1 => Address::ViaPair(D, E);
    indirect_hl_increment: 2 => Address::ViaHlStep(1);
    indirect_hl_decrement: 3 => Address::ViaHlStep(-1);
}

k::equations! {
    for AccumulatorTable;
    use Unary::*;
    accumulator_rlca: 0 => Rlca;
    accumulator_rrca: 1 => Rrca;
    accumulator_rla:  2 => Rla;
    accumulator_rra:  3 => Rra;
    accumulator_daa:  4 => Daa;
    accumulator_cpl:  5 => Cpl;
    accumulator_scf:  6 => Scf;
    accumulator_ccf:  7 => Ccf;
}

k::equations! {
    for Cc;
    use Condition::*;
    cc_nz: 0 => NotZero;
    cc_z:  1 => Zero;
    cc_nc: 2 => NotCarry;
    cc_c:  3 => Carry;
}

k::equations! {
    for AluTable;
    use Alu::*;
    alu_add: 0 => Add;
    alu_adc: 1 => Adc;
    alu_sub: 2 => Sub;
    alu_sbc: 3 => Sbc;
    alu_and: 4 => And;
    alu_xor: 5 => Xor;
    alu_or:  6 => Or;
    alu_cp:  7 => Cp;
}

k::equations! {
    for Rot;
    use Unary::*;
    rot_rlc:  0 => Rlc;
    rot_rrc:  1 => Rrc;
    rot_rl:   2 => Rl;
    rot_rr:   3 => Rr;
    rot_sla:  4 => Sla;
    rot_sra:  5 => Sra;
    rot_swap: 6 => Swap;
    rot_srl:  7 => Srl;
}
