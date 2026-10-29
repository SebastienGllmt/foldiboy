//! Opcode-layout contracts from Pan Docs CPU Instruction Set and gbops dmgops.json.
//! Native finite alternatives cover every byte on each page, before instruction execution.
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

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_ld_r16_immediate: {
            k: [crate::Decode(opcode @ (0x01 | 0x11 | 0x21 | 0x31))],
        } => {
            k: [Instruction::Ld16 { pair: ExpectedPair(opcode / 0x10) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_store_a_indirect: {
            k: [crate::Decode(opcode @ (0x02 | 0x12 | 0x22 | 0x32))],
        } => {
            k: [Instruction::StoreA { addr: ExpectedIndirectAddress(opcode / 0x10) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_load_a_indirect: {
            k: [crate::Decode(opcode @ (0x0A | 0x1A | 0x2A | 0x3A))],
        } => {
            k: [Instruction::LoadA { addr: ExpectedIndirectAddress(opcode / 0x10) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_inc16: {
            k: [crate::Decode(opcode @ (0x03 | 0x13 | 0x23 | 0x33))],
        } => {
            k: [Instruction::Inc16 { pair: ExpectedPair(opcode / 0x10) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_dec16: {
            k: [crate::Decode(opcode @ (0x0B | 0x1B | 0x2B | 0x3B))],
        } => {
            k: [Instruction::Dec16 { pair: ExpectedPair(opcode / 0x10) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_addhl: {
            k: [crate::Decode(opcode @ (0x09 | 0x19 | 0x29 | 0x39))],
        } => {
            k: [Instruction::AddHl { pair: ExpectedPair(opcode / 0x10) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_inc_r8: {
            k: [crate::Decode(opcode @ (0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C))],
        } => {
            k: [Instruction::Update8 { op: Unary::Inc, target: ExpectedOperand(opcode / 8) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_dec_r8: {
            k: [crate::Decode(opcode @ (0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D))],
        } => {
            k: [Instruction::Update8 { op: Unary::Dec, target: ExpectedOperand(opcode / 8) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_ld_r8_immediate: {
            k: [crate::Decode(opcode @ (0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E))],
        } => {
            k: [Instruction::Ld8 { dst: ExpectedOperand(opcode / 8), src: Operand::Immediate }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_accumulator: {
            k: [crate::Decode(opcode @ (0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F))],
        } => {
            k: [
                Instruction::Update8 {
                    op: ExpectedAccumulatorOp(opcode / 8),
                    target: Operand::Reg { r: Register::A },
                },
            ],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_jr_conditional: {
            k: [crate::Decode(opcode @ (0x20 | 0x28 | 0x30 | 0x38))],
        } => {
            k: [Instruction::JumpRelative { cond: ExpectedCondition((opcode / 8) % 4) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_ld_r8_r8: {
            k: [
                crate::Decode(
                    opcode @ (0x40
                    | 0x41
                    | 0x42
                    | 0x43
                    | 0x44
                    | 0x45
                    | 0x46
                    | 0x47
                    | 0x48
                    | 0x49
                    | 0x4A
                    | 0x4B
                    | 0x4C
                    | 0x4D
                    | 0x4E
                    | 0x4F
                    | 0x50
                    | 0x51
                    | 0x52
                    | 0x53
                    | 0x54
                    | 0x55
                    | 0x56
                    | 0x57
                    | 0x58
                    | 0x59
                    | 0x5A
                    | 0x5B
                    | 0x5C
                    | 0x5D
                    | 0x5E
                    | 0x5F
                    | 0x60
                    | 0x61
                    | 0x62
                    | 0x63
                    | 0x64
                    | 0x65
                    | 0x66
                    | 0x67
                    | 0x68
                    | 0x69
                    | 0x6A
                    | 0x6B
                    | 0x6C
                    | 0x6D
                    | 0x6E
                    | 0x6F
                    | 0x70
                    | 0x71
                    | 0x72
                    | 0x73
                    | 0x74
                    | 0x75
                    | 0x77
                    | 0x78
                    | 0x79
                    | 0x7A
                    | 0x7B
                    | 0x7C
                    | 0x7D
                    | 0x7E
                    | 0x7F),
                ),
            ],
        } => {
            k: [
                Instruction::Ld8 {
                    dst: ExpectedOperand((opcode / 8) % 8),
                    src: ExpectedOperand(opcode % 8),
                },
            ],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_alu_r8: {
            k: [
                crate::Decode(
                    opcode @ (0x80
                    | 0x81
                    | 0x82
                    | 0x83
                    | 0x84
                    | 0x85
                    | 0x86
                    | 0x87
                    | 0x88
                    | 0x89
                    | 0x8A
                    | 0x8B
                    | 0x8C
                    | 0x8D
                    | 0x8E
                    | 0x8F
                    | 0x90
                    | 0x91
                    | 0x92
                    | 0x93
                    | 0x94
                    | 0x95
                    | 0x96
                    | 0x97
                    | 0x98
                    | 0x99
                    | 0x9A
                    | 0x9B
                    | 0x9C
                    | 0x9D
                    | 0x9E
                    | 0x9F
                    | 0xA0
                    | 0xA1
                    | 0xA2
                    | 0xA3
                    | 0xA4
                    | 0xA5
                    | 0xA6
                    | 0xA7
                    | 0xA8
                    | 0xA9
                    | 0xAA
                    | 0xAB
                    | 0xAC
                    | 0xAD
                    | 0xAE
                    | 0xAF
                    | 0xB0
                    | 0xB1
                    | 0xB2
                    | 0xB3
                    | 0xB4
                    | 0xB5
                    | 0xB6
                    | 0xB7
                    | 0xB8
                    | 0xB9
                    | 0xBA
                    | 0xBB
                    | 0xBC
                    | 0xBD
                    | 0xBE
                    | 0xBF),
                ),
            ],
        } => {
            k: [
                Instruction::Alu8 {
                    op: ExpectedAluOp((opcode / 8) % 8),
                    src: ExpectedOperand(opcode % 8),
                },
            ],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_returnif: {
            k: [crate::Decode(opcode @ (0xC0 | 0xC8 | 0xD0 | 0xD8))],
        } => {
            k: [Instruction::ReturnIf { cond: ExpectedCondition((opcode / 8) % 4) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_jump: {
            k: [crate::Decode(opcode @ (0xC2 | 0xCA | 0xD2 | 0xDA))],
        } => {
            k: [Instruction::Jump { cond: ExpectedCondition((opcode / 8) % 4) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_call: {
            k: [crate::Decode(opcode @ (0xC4 | 0xCC | 0xD4 | 0xDC))],
        } => {
            k: [Instruction::Call { cond: ExpectedCondition((opcode / 8) % 4) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_pop: {
            k: [crate::Decode(opcode @ (0xC1 | 0xD1 | 0xE1 | 0xF1))],
        } => {
            k: [Instruction::Pop { pair: ExpectedStackPair((opcode / 0x10) % 4) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_push: {
            k: [crate::Decode(opcode @ (0xC5 | 0xD5 | 0xE5 | 0xF5))],
        } => {
            k: [Instruction::Push { pair: ExpectedStackPair((opcode / 0x10) % 4) }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_alu_immediate: {
            k: [crate::Decode(opcode @ (0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE))],
        } => {
            k: [Instruction::Alu8 { op: ExpectedAluOp((opcode / 8) % 8), src: Operand::Immediate }],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
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

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_cb_rotations: {
            k: [
                crate::DecodeCb(
                    opcode @ (0x00
                    | 0x01
                    | 0x02
                    | 0x03
                    | 0x04
                    | 0x05
                    | 0x06
                    | 0x07
                    | 0x08
                    | 0x09
                    | 0x0A
                    | 0x0B
                    | 0x0C
                    | 0x0D
                    | 0x0E
                    | 0x0F
                    | 0x10
                    | 0x11
                    | 0x12
                    | 0x13
                    | 0x14
                    | 0x15
                    | 0x16
                    | 0x17
                    | 0x18
                    | 0x19
                    | 0x1A
                    | 0x1B
                    | 0x1C
                    | 0x1D
                    | 0x1E
                    | 0x1F
                    | 0x20
                    | 0x21
                    | 0x22
                    | 0x23
                    | 0x24
                    | 0x25
                    | 0x26
                    | 0x27
                    | 0x28
                    | 0x29
                    | 0x2A
                    | 0x2B
                    | 0x2C
                    | 0x2D
                    | 0x2E
                    | 0x2F
                    | 0x30
                    | 0x31
                    | 0x32
                    | 0x33
                    | 0x34
                    | 0x35
                    | 0x36
                    | 0x37
                    | 0x38
                    | 0x39
                    | 0x3A
                    | 0x3B
                    | 0x3C
                    | 0x3D
                    | 0x3E
                    | 0x3F),
                ),
            ],
        } => {
            k: [
                Instruction::Update8 {
                    op: ExpectedRotateOp(opcode / 8),
                    target: ExpectedOperand(opcode % 8),
                },
            ],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_cb_bit: {
            k: [
                crate::DecodeCb(
                    opcode @ (0x40
                    | 0x41
                    | 0x42
                    | 0x43
                    | 0x44
                    | 0x45
                    | 0x46
                    | 0x47
                    | 0x48
                    | 0x49
                    | 0x4A
                    | 0x4B
                    | 0x4C
                    | 0x4D
                    | 0x4E
                    | 0x4F
                    | 0x50
                    | 0x51
                    | 0x52
                    | 0x53
                    | 0x54
                    | 0x55
                    | 0x56
                    | 0x57
                    | 0x58
                    | 0x59
                    | 0x5A
                    | 0x5B
                    | 0x5C
                    | 0x5D
                    | 0x5E
                    | 0x5F
                    | 0x60
                    | 0x61
                    | 0x62
                    | 0x63
                    | 0x64
                    | 0x65
                    | 0x66
                    | 0x67
                    | 0x68
                    | 0x69
                    | 0x6A
                    | 0x6B
                    | 0x6C
                    | 0x6D
                    | 0x6E
                    | 0x6F
                    | 0x70
                    | 0x71
                    | 0x72
                    | 0x73
                    | 0x74
                    | 0x75
                    | 0x76
                    | 0x77
                    | 0x78
                    | 0x79
                    | 0x7A
                    | 0x7B
                    | 0x7C
                    | 0x7D
                    | 0x7E
                    | 0x7F),
                ),
            ],
        } => {
            k: [
                Instruction::TestBit { bit: (opcode / 8) % 8, target: ExpectedOperand(opcode % 8) },
            ],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_cb_reset: {
            k: [
                crate::DecodeCb(
                    opcode @ (0x80
                    | 0x81
                    | 0x82
                    | 0x83
                    | 0x84
                    | 0x85
                    | 0x86
                    | 0x87
                    | 0x88
                    | 0x89
                    | 0x8A
                    | 0x8B
                    | 0x8C
                    | 0x8D
                    | 0x8E
                    | 0x8F
                    | 0x90
                    | 0x91
                    | 0x92
                    | 0x93
                    | 0x94
                    | 0x95
                    | 0x96
                    | 0x97
                    | 0x98
                    | 0x99
                    | 0x9A
                    | 0x9B
                    | 0x9C
                    | 0x9D
                    | 0x9E
                    | 0x9F
                    | 0xA0
                    | 0xA1
                    | 0xA2
                    | 0xA3
                    | 0xA4
                    | 0xA5
                    | 0xA6
                    | 0xA7
                    | 0xA8
                    | 0xA9
                    | 0xAA
                    | 0xAB
                    | 0xAC
                    | 0xAD
                    | 0xAE
                    | 0xAF
                    | 0xB0
                    | 0xB1
                    | 0xB2
                    | 0xB3
                    | 0xB4
                    | 0xB5
                    | 0xB6
                    | 0xB7
                    | 0xB8
                    | 0xB9
                    | 0xBA
                    | 0xBB
                    | 0xBC
                    | 0xBD
                    | 0xBE
                    | 0xBF),
                ),
            ],
        } => {
            k: [
                Instruction::Update8 {
                    op: Unary::ResetBit { bit: (opcode / 8) % 8 },
                    target: ExpectedOperand(opcode % 8),
                },
            ],
        };

        /// Pan Docs opcode layout: explicit finite byte alternatives and independently specified fields.
        #[k::claim(all_paths)]
        contract_decode_cb_set: {
            k: [
                crate::DecodeCb(
                    opcode @ (0xC0
                    | 0xC1
                    | 0xC2
                    | 0xC3
                    | 0xC4
                    | 0xC5
                    | 0xC6
                    | 0xC7
                    | 0xC8
                    | 0xC9
                    | 0xCA
                    | 0xCB
                    | 0xCC
                    | 0xCD
                    | 0xCE
                    | 0xCF
                    | 0xD0
                    | 0xD1
                    | 0xD2
                    | 0xD3
                    | 0xD4
                    | 0xD5
                    | 0xD6
                    | 0xD7
                    | 0xD8
                    | 0xD9
                    | 0xDA
                    | 0xDB
                    | 0xDC
                    | 0xDD
                    | 0xDE
                    | 0xDF
                    | 0xE0
                    | 0xE1
                    | 0xE2
                    | 0xE3
                    | 0xE4
                    | 0xE5
                    | 0xE6
                    | 0xE7
                    | 0xE8
                    | 0xE9
                    | 0xEA
                    | 0xEB
                    | 0xEC
                    | 0xED
                    | 0xEE
                    | 0xEF
                    | 0xF0
                    | 0xF1
                    | 0xF2
                    | 0xF3
                    | 0xF4
                    | 0xF5
                    | 0xF6
                    | 0xF7
                    | 0xF8
                    | 0xF9
                    | 0xFA
                    | 0xFB
                    | 0xFC
                    | 0xFD
                    | 0xFE
                    | 0xFF),
                ),
            ],
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
