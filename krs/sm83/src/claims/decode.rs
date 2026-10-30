//! Opcode-layout contracts from Pan Docs CPU Instruction Set and gbops dmgops.json.
//! Opcode-family conditions and finite cases cover every byte on each page, before execution.
//! Behavior: RGBDS gbz80(7), https://rgbds.gbdev.io/docs/gbz80.7 .
//! Encoding/families: Pan Docs CPU Instruction Set; bus order: izik1/gbops dmgops.json.
//! See README.md for pinned sources, coverage and model exclusions.
//! These are authored proof inputs; none calls the implementation's ALU/result helpers.

use crate::Machine;
use crate::alu::{Alu, AluOp, Unary, UnaryOp};
use crate::decode::{
    AddrMode, Address, Cond, Condition, Instruction, Operand, Operand8, Pair16, RegPair,
};
use crate::machine::{Cycle, Register};
use k::builtin::Int;

/// HALT occupies the register-load family's LD (HL),(HL) encoding.
#[k::constant]
pub const HALT_OPCODE: Int = 0x76;

/// Pan Docs encoding table; independent of the implementation operand tables.
#[k::function(total, internal)]
pub fn ExpectedPair(index: k::Term<Int>) -> k::Term<RegPair> {
    match index {
        0 => Pair16::Registers {
            high: Register::B,
            low: Register::C,
        },
        1 => Pair16::Registers {
            high: Register::D,
            low: Register::E,
        },
        2 => Pair16::Registers {
            high: Register::H,
            low: Register::L,
        },
        _ => Pair16::StackPointer,
    }
}

/// Pan Docs encoding table; independent of the implementation operand tables.
#[k::function(total, internal)]
pub fn ExpectedIndirectAddress(index: k::Term<Int>) -> k::Term<AddrMode> {
    match index {
        0 => Address::ViaPair {
            high: Register::B,
            low: Register::C,
        },
        1 => Address::ViaPair {
            high: Register::D,
            low: Register::E,
        },
        2 => Address::ViaHlStep { delta: 1 },
        _ => Address::ViaHlStep { delta: -1 },
    }
}

/// Pan Docs encoding table; independent of the implementation operand tables.
#[k::function(total, internal)]
pub fn ExpectedOperand(index: k::Term<Int>) -> k::Term<Operand8> {
    match index {
        0 => Operand::Reg { r: Register::B },
        1 => Operand::Reg { r: Register::C },
        2 => Operand::Reg { r: Register::D },
        3 => Operand::Reg { r: Register::E },
        4 => Operand::Reg { r: Register::H },
        5 => Operand::Reg { r: Register::L },
        6 => Operand::HlIndirect,
        _ => Operand::Reg { r: Register::A },
    }
}

/// Pan Docs encoding table; independent of the implementation operand tables.
#[k::function(total, internal)]
pub fn ExpectedAccumulatorOp(index: k::Term<Int>) -> k::Term<UnaryOp> {
    match index {
        0 => Unary::Rlca,
        1 => Unary::Rrca,
        2 => Unary::Rla,
        3 => Unary::Rra,
        4 => Unary::Daa,
        5 => Unary::Cpl,
        6 => Unary::Scf,
        _ => Unary::Ccf,
    }
}

/// Pan Docs encoding table; independent of the implementation operand tables.
#[k::function(total, internal)]
pub fn ExpectedCondition(index: k::Term<Int>) -> k::Term<Cond> {
    match index {
        0 => Condition::NotZero,
        1 => Condition::Zero,
        2 => Condition::NotCarry,
        _ => Condition::Carry,
    }
}

/// Pan Docs encoding table; independent of the implementation operand tables.
#[k::function(total, internal)]
pub fn ExpectedAluOp(index: k::Term<Int>) -> k::Term<AluOp> {
    match index {
        0 => Alu::Add,
        1 => Alu::Adc,
        2 => Alu::Sub,
        3 => Alu::Sbc,
        4 => Alu::And,
        5 => Alu::Xor,
        6 => Alu::Or,
        _ => Alu::Cp,
    }
}

/// Pan Docs encoding table; independent of the implementation operand tables.
#[k::function(total, internal)]
pub fn ExpectedStackPair(index: k::Term<Int>) -> k::Term<RegPair> {
    match index {
        0 => Pair16::Registers {
            high: Register::B,
            low: Register::C,
        },
        1 => Pair16::Registers {
            high: Register::D,
            low: Register::E,
        },
        2 => Pair16::Registers {
            high: Register::H,
            low: Register::L,
        },
        _ => Pair16::Af,
    }
}

/// Pan Docs encoding table; independent of the implementation operand tables.
#[k::function(total, internal)]
pub fn ExpectedRotateOp(index: k::Term<Int>) -> k::Term<UnaryOp> {
    match index {
        0 => Unary::Rlc,
        1 => Unary::Rrc,
        2 => Unary::Rl,
        3 => Unary::Rr,
        4 => Unary::Sla,
        5 => Unary::Sra,
        6 => Unary::Swap,
        _ => Unary::Srl,
    }
}

#[k::spec_module(name = "SM83-DECODE-SPEC", imports = [crate])]
pub mod spec {
    use super::*;
    k::claims! {
        for Machine;

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 __0 001"))]
        contract_decode_ld_r16_immediate: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::Ld16 { pair: ExpectedPair((opcode / 0x10) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 __0 010"))]
        contract_decode_store_a_indirect: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::StoreA { addr: ExpectedIndirectAddress((opcode / 0x10) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 __1 010"))]
        contract_decode_load_a_indirect: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::LoadA { addr: ExpectedIndirectAddress((opcode / 0x10) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 __0 011"))]
        contract_decode_inc16: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::Inc16 { pair: ExpectedPair((opcode / 0x10) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 __1 011"))]
        contract_decode_dec16: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::Dec16 { pair: ExpectedPair((opcode / 0x10) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 __1 001"))]
        contract_decode_addhl: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::AddHl { pair: ExpectedPair((opcode / 0x10) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 ___ 100"))]
        contract_decode_inc_r8: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::Update8 { op: Unary::Inc, target: ExpectedOperand((opcode / 8) % 8) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 ___ 101"))]
        contract_decode_dec_r8: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::Update8 { op: Unary::Dec, target: ExpectedOperand((opcode / 8) % 8) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 ___ 110"))]
        contract_decode_ld_r8_immediate: {
            k: [crate::Decode(opcode)],
        } => {
            k: [
                Instruction::Ld8 {
                    dst: ExpectedOperand((opcode / 8) % 8),
                    src: Operand::Immediate,
                },
            ],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 ___ 111"))]
        contract_decode_accumulator: {
            k: [crate::Decode(opcode)],
        } => {
            k: [
                Instruction::Update8 {
                    op: ExpectedAccumulatorOp((opcode / 8) % 8),
                    target: Operand::Reg { r: Register::A },
                },
            ],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 1__ 000"))]
        contract_decode_jr_conditional: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::JumpRelative { cond: ExpectedCondition((opcode / 8) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "01 ___ ___") && opcode != HALT_OPCODE)]
        contract_decode_ld_r8_r8: {
            k: [crate::Decode(opcode)],
        } => {
            k: [
                Instruction::Ld8 {
                    dst: ExpectedOperand((opcode / 8) % 8),
                    src: ExpectedOperand(opcode % 8),
                },
            ],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "10 ___ ___"))]
        contract_decode_alu_r8: {
            k: [crate::Decode(opcode)],
        } => {
            k: [
                Instruction::Alu8 {
                    op: ExpectedAluOp((opcode / 8) % 8),
                    src: ExpectedOperand(opcode % 8),
                },
            ],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "11 0__ 000"))]
        contract_decode_returnif: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::ReturnIf { cond: ExpectedCondition((opcode / 8) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "11 0__ 010"))]
        contract_decode_jump: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::Jump { cond: ExpectedCondition((opcode / 8) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "11 0__ 100"))]
        contract_decode_call: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::Call { cond: ExpectedCondition((opcode / 8) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "11 __0 001"))]
        contract_decode_pop: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::Pop { pair: ExpectedStackPair((opcode / 0x10) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "11 __0 101"))]
        contract_decode_push: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::Push { pair: ExpectedStackPair((opcode / 0x10) % 4) }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "11 ___ 110"))]
        contract_decode_alu_immediate: {
            k: [crate::Decode(opcode)],
        } => {
            k: [Instruction::Alu8 { op: ExpectedAluOp((opcode / 8) % 8), src: Operand::Immediate }],
        };

        /// RST retains explicit cases: the generated Lean closer does not yet handle its symbolic arithmetic target.
        #[k::claim(all_paths)]
        contract_decode_rst: {
            k: [crate::Decode(opcode @ (0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF))],
        } => {
            k: [Instruction::Restart { target: ((opcode / 8) % 8) * 8 }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_00: {
            k: [crate::Decode(0x00)],
        } => {
            k: [Instruction::Nop],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_08: {
            k: [crate::Decode(0x08)],
        } => {
            k: [Instruction::StoreSp],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_18: {
            k: [crate::Decode(0x18)],
        } => {
            k: [Instruction::JumpRelative { cond: Condition::Always }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_76: {
            k: [crate::Decode(0x76)],
        } => {
            k: [Instruction::Halt],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_c3: {
            k: [crate::Decode(0xC3)],
        } => {
            k: [Instruction::Jump { cond: Condition::Always }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_c9: {
            k: [crate::Decode(0xC9)],
        } => {
            k: [Instruction::Return],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_cb: {
            k: [crate::Decode(0xCB)],
        } => {
            k: [Instruction::Prefix],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_cd: {
            k: [crate::Decode(0xCD)],
        } => {
            k: [Instruction::Call { cond: Condition::Always }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_d9: {
            k: [crate::Decode(0xD9)],
        } => {
            k: [Instruction::ReturnFromInterrupt],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_e0: {
            k: [crate::Decode(0xE0)],
        } => {
            k: [Instruction::StoreA { addr: Address::HighImmediate }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_e2: {
            k: [crate::Decode(0xE2)],
        } => {
            k: [Instruction::StoreA { addr: Address::HighC }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_e8: {
            k: [crate::Decode(0xE8)],
        } => {
            k: [Instruction::AddSpOffset],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_e9: {
            k: [crate::Decode(0xE9)],
        } => {
            k: [Instruction::JumpHl],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_ea: {
            k: [crate::Decode(0xEA)],
        } => {
            k: [Instruction::StoreA { addr: Address::Absolute }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_f0: {
            k: [crate::Decode(0xF0)],
        } => {
            k: [Instruction::LoadA { addr: Address::HighImmediate }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_f2: {
            k: [crate::Decode(0xF2)],
        } => {
            k: [Instruction::LoadA { addr: Address::HighC }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_f3: {
            k: [crate::Decode(0xF3)],
        } => {
            k: [Instruction::DisableInterrupts],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_f8: {
            k: [crate::Decode(0xF8)],
        } => {
            k: [Instruction::LdHlSpOffset],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_f9: {
            k: [crate::Decode(0xF9)],
        } => {
            k: [Instruction::LdSpHl],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_fa: {
            k: [crate::Decode(0xFA)],
        } => {
            k: [Instruction::LoadA { addr: Address::Absolute }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_fixed_fb: {
            k: [crate::Decode(0xFB)],
        } => {
            k: [Instruction::EnableInterrupts],
        };

        /// Model policy only: STOP is unimplemented; unused opcodes have no execution rule. This is not a hardware STOP claim.
        #[k::claim(all_paths)]
        contract_decode_unsupported_policy: {
            k: [
                crate::Decode(
                    opcode @ (0x10
                    | 0xD3
                    | 0xDB
                    | 0xDD
                    | 0xE3
                    | 0xE4
                    | 0xEB
                    | 0xEC
                    | 0xED
                    | 0xF4
                    | 0xFC
                    | 0xFD),
                ),
            ],
        } => {
            k: [Instruction::Unimplemented { opcode }],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "00 ___ ___"))]
        contract_decode_cb_rotations: {
            k: [crate::DecodeCb(opcode)],
        } => {
            k: [
                Instruction::Update8 {
                    op: ExpectedRotateOp((opcode / 8) % 8),
                    target: ExpectedOperand(opcode % 8),
                },
            ],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "01 ___ ___"))]
        contract_decode_cb_bit: {
            k: [crate::DecodeCb(opcode)],
        } => {
            k: [
                Instruction::TestBit { bit: (opcode / 8) % 8, target: ExpectedOperand(opcode % 8) },
            ],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "10 ___ ___"))]
        contract_decode_cb_reset: {
            k: [crate::DecodeCb(opcode)],
        } => {
            k: [
                Instruction::Update8 {
                    op: Unary::ResetBit { bit: (opcode / 8) % 8 },
                    target: ExpectedOperand(opcode % 8),
                },
            ],
        };

        /// Pan Docs opcode family and independently specified decoded fields.
        #[k::claim(all_paths)]
        #[k::requires(k::bits!(opcode, "11 ___ ___"))]
        contract_decode_cb_set: {
            k: [crate::DecodeCb(opcode)],
        } => {
            k: [
                Instruction::Update8 {
                    op: Unary::SetBit { bit: (opcode / 8) % 8 },
                    target: ExpectedOperand(opcode % 8),
                },
            ],
        };
    }
}
