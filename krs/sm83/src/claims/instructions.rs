//! Decoded instruction contracts, after opcode fetch and interrupt arbitration.
//! Behavior: RGBDS gbz80(7), https://rgbds.gbdev.io/docs/gbz80.7 .
//! Encoding/families: Pan Docs CPU Instruction Set; bus order: izik1/gbops dmgops.json.
//! See README.md for pinned sources, coverage and model exclusions.
//! These are authored proof inputs; none calls the implementation's ALU/result helpers.
//! SUB/SBC/CP add two byte moduli (0x200) before `% 0x100` to keep
//! the remainder's input nonnegative throughout the stated byte domain.

use crate::Machine;
use crate::alu::{Alu, Flags, Unary};
use crate::decode::{Address, Condition, Instruction, Operand, Pair16};
use crate::interrupt::Step;
use crate::machine::{Event, Register};
use k::builtin::{Bool, Int, int, kequal};

/// RGBDS DAA: adjust the high digit after a carry or an addition above BCD 99.
#[k::function(total, internal)]
pub fn ExpectedDaaHigh(
    a: k::Term<Int>,
    subtract: k::Term<Bool>,
    carry: k::Term<Bool>,
) -> k::Term<Int> {
    kequal::ite(carry || (!subtract && a > 0x99), 0x60, 0)
}

/// RGBDS DAA: adjust the low digit after a half-carry or an invalid added digit.
#[k::function(total, internal)]
pub fn ExpectedDaaLow(
    a: k::Term<Int>,
    subtract: k::Term<Bool>,
    half_carry: k::Term<Bool>,
) -> k::Term<Int> {
    kequal::ite(half_carry || (!subtract && a % 0x10 > 9), 0x06, 0)
}

/// Apply the correction in the direction of the preceding arithmetic operation.
#[k::function(total, internal)]
pub fn ExpectedDaaCorrection(
    a: k::Term<Int>,
    subtract: k::Term<Bool>,
    half_carry: k::Term<Bool>,
    carry: k::Term<Bool>,
) -> k::Term<Int> {
    kequal::ite(
        !subtract,
        ExpectedDaaHigh(a, subtract, carry) + ExpectedDaaLow(a, subtract, half_carry),
        -(ExpectedDaaHigh(a, subtract, carry) + ExpectedDaaLow(a, subtract, half_carry)),
    )
}

/// Independent DAA byte expectation; keep subtraction nonnegative before reduction.
#[k::function(total, internal)]
pub fn ExpectedDaaByte(
    a: k::Term<Int>,
    subtract: k::Term<Bool>,
    half_carry: k::Term<Bool>,
    carry: k::Term<Bool>,
) -> k::Term<Int> {
    (a + 0x100 + ExpectedDaaCorrection(a, subtract, half_carry, carry)) % 0x100
}

#[k::spec_module(name = "SM83-INSTRUCTIONS-SPEC", imports = [crate])]
pub mod spec {
    use super::*;
    k::claims! {
        for Machine;

        /// NOP: no architectural state change or remaining bus cycle.
        #[k::claim(all_paths)]
        contract_nop: {
            k: [Instruction::Nop],
        } => {
            k: [],
        };

        /// Pan Docs: an unused opcode locks up the CPU; no further step follows.
        #[k::claim(all_paths)]
        contract_illegal_locks: {
            k: [Instruction::Illegal { opcode }],
        } => {
            k: [Step::Locked],
        };

        /// LD r8,source: load one byte, preserve flags and all other registers.
        #[k::claim(all_paths)]
        #[k::requires((0 <= r[src] && r[src] < 0x100))]
        contract_ld8_register: {
            k: [Instruction::Ld8 { dst: Operand::Reg { r: dst }, src: Operand::Reg { r: src } }],
            regs: r,
            z: _,
        } => {
            k: [],
            regs: k::map![dst -> r[src], ..r],
            z: r[src],
        };

        /// LD r8,source: load one byte, preserve flags and all other registers.
        #[k::claim(all_paths)]
        #[k::requires((0 <= pc && pc < 0x1_0000) && (0 <= m[pc] && m[pc] < 0x100))]
        contract_ld8_immediate: {
            k: [Instruction::Ld8 { dst: Operand::Reg { r: dst }, src: Operand::Immediate }],
            regs: r,
            z: _,
            pc: pc,
            memory: m,
            bus: log,
        } => {
            k: [],
            regs: k::map![dst -> m[pc], ..r],
            z: m[pc],
            pc: ((pc + 1) % 0x1_0000),
            bus: [..log, Event::Read { addr: pc, value: m[pc] }],
        };

        /// LD r8,source: load one byte, preserve flags and all other registers.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100)
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100),
        )]
        contract_ld8_indirect: {
            k: [Instruction::Ld8 { dst: Operand::Reg { r: dst }, src: Operand::HlIndirect }],
            regs: r,
            z: _,
            memory: m,
            bus: log,
        } => {
            k: [],
            regs: k::map![dst -> m[r[Register::H] * 0x100 + r[Register::L]], ..r],
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// LD [HL],source: one byte write; immediate is read before the write, including aliasing.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100)
                && (0 <= r[src] && r[src] < 0x100)
                && m.contains_key(r[Register::H] * 0x100 + r[Register::L])
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100),
        )]
        contract_ld8_store_register: {
            k: [Instruction::Ld8 { dst: Operand::HlIndirect, src: Operand::Reg { r: src } }],
            regs: r,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            memory: k::map![(r[Register::H] * 0x100 + r[Register::L]) -> r[src], ..m],
            z: r[src],
            bus: [
                ..log,
                Event::Write {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: r[src],
                },
            ],
        };

        /// LD [HL],source: one byte write; immediate is read before the write, including aliasing.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100)
                && (0 <= m[pc] && m[pc] < 0x100)
                && m.contains_key(r[Register::H] * 0x100 + r[Register::L])
                && (0 <= pc && pc < 0x1_0000)
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100),
        )]
        contract_ld8_store_immediate: {
            k: [Instruction::Ld8 { dst: Operand::HlIndirect, src: Operand::Immediate }],
            regs: r,
            memory: m,
            z: _,
            bus: log,
            pc: pc,
        } => {
            k: [],
            memory: k::map![(r[Register::H] * 0x100 + r[Register::L]) -> m[pc], ..m],
            z: m[pc],
            bus: [
                ..log,
                Event::Read { addr: pc, value: m[pc] },
                Event::Write {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[pc],
                },
            ],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// LD r16,d16: little-endian immediate reads, including PC wrapping.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && (0 <= m[pc] && m[pc] < 0x100)
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100),
        )]
        contract_ld16_registers: {
            k: [Instruction::Ld16 { pair: Pair16::Registers { high, low } }],
            pc: pc,
            memory: m,
            w: _,
            z: _,
            bus: log,
            regs: r,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            w: m[(pc + 1) % 0x1_0000],
            z: m[pc],
            bus: [
                ..log,
                Event::Read { addr: pc, value: m[pc] },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
            ],
            regs: k::map![high -> m[(pc + 1) % 0x1_0000], low -> m[pc], ..r],
        };

        /// LD r16,d16: little-endian immediate reads, including PC wrapping.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && (0 <= m[pc] && m[pc] < 0x100)
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100),
        )]
        contract_ld16_sp: {
            k: [Instruction::Ld16 { pair: Pair16::StackPointer }],
            pc: pc,
            memory: m,
            w: _,
            z: _,
            bus: log,
            sp: _,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            w: m[(pc + 1) % 0x1_0000],
            z: m[pc],
            bus: [
                ..log,
                Event::Read { addr: pc, value: m[pc] },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
            ],
            sp: (m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]),
        };

        /// LD SP,HL: preserve flags and take one internal cycle.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_ld_sp_hl: {
            k: [Instruction::LdSpHl],
            regs: r,
            sp: _,
            bus: log,
        } => {
            k: [],
            sp: (r[Register::H] * 0x100 + r[Register::L]),
            bus: [..log, Event::Idle],
        };

        /// ADD A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= r[src] && r[src] < 0x100),
        )]
        contract_alu_add_register: {
            k: [Instruction::Alu8 { op: Alu::Add, src: Operand::Reg { r: src } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            regs: k::map![Register::A -> (((r[Register::A] + r[src] + 0)) % 0x100), ..r],
            f: Flags {
                z: (r[Register::A] + r[src]) % 0x100 == 0,
                n: false,
                h: r[Register::A] % 0x10 + r[src] % 0x10 >= 0x10,
                c: r[Register::A] + r[src] >= 0x100,
            },
            z: r[src],
            bus: log,
        };

        /// ADD A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[pc] && m[pc] < 0x100)
                && (0 <= pc && pc < 0x1_0000),
        )]
        contract_alu_add_immediate: {
            k: [Instruction::Alu8 { op: Alu::Add, src: Operand::Immediate }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
            pc: pc,
        } => {
            k: [],
            regs: k::map![Register::A -> (((r[Register::A] + m[pc] + 0)) % 0x100), ..r],
            f: Flags {
                z: (r[Register::A] + m[pc]) % 0x100 == 0,
                n: false,
                h: r[Register::A] % 0x10 + m[pc] % 0x10 >= 0x10,
                c: r[Register::A] + m[pc] >= 0x100,
            },
            z: m[pc],
            bus: [..log, Event::Read { addr: pc, value: m[pc] }],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// ADD A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_alu_add_indirect: {
            k: [Instruction::Alu8 { op: Alu::Add, src: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            regs: k::map![
                Register::A -> (
                    ((r[Register::A] + m[r[Register::H] * 0x100 + r[Register::L]] + 0)) % 0x100
                ),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] + m[r[Register::H] * 0x100 + r[Register::L]]) % 0x100 == 0,
                n: false,
                h: r[Register::A] % 0x10 + m[r[Register::H] * 0x100 + r[Register::L]] % 0x10 >= 0x10,
                c: r[Register::A] + m[r[Register::H] * 0x100 + r[Register::L]] >= 0x100,
            },
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// ADC A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= r[src] && r[src] < 0x100),
        )]
        contract_alu_adc_register: {
            k: [Instruction::Alu8 { op: Alu::Adc, src: Operand::Reg { r: src } }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            regs: k::map![
                Register::A -> (((r[Register::A] + r[src] + kequal::ite(carry, 1, 0))) % 0x100),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] + r[src] + kequal::ite(carry, 1, 0)) % 0x100 == 0,
                n: false,
                h: r[Register::A] % 0x10 + r[src] % 0x10 + kequal::ite(carry, 1, 0) >= 0x10,
                c: r[Register::A] + r[src] + kequal::ite(carry, 1, 0) >= 0x100,
            },
            z: r[src],
            bus: log,
        };

        /// ADC A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[pc] && m[pc] < 0x100)
                && (0 <= pc && pc < 0x1_0000),
        )]
        contract_alu_adc_immediate: {
            k: [Instruction::Alu8 { op: Alu::Adc, src: Operand::Immediate }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
            memory: m,
            pc: pc,
        } => {
            k: [],
            regs: k::map![
                Register::A -> (((r[Register::A] + m[pc] + kequal::ite(carry, 1, 0))) % 0x100),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] + m[pc] + kequal::ite(carry, 1, 0)) % 0x100 == 0,
                n: false,
                h: r[Register::A] % 0x10 + m[pc] % 0x10 + kequal::ite(carry, 1, 0) >= 0x10,
                c: r[Register::A] + m[pc] + kequal::ite(carry, 1, 0) >= 0x100,
            },
            z: m[pc],
            bus: [..log, Event::Read { addr: pc, value: m[pc] }],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// ADC A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_alu_adc_indirect: {
            k: [Instruction::Alu8 { op: Alu::Adc, src: Operand::HlIndirect }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            regs: k::map![
                Register::A -> ((r[Register::A] + m[r[Register::H] * 0x100 + r[Register::L]] + kequal::ite(carry, 1, 0)) % 0x100),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] + m[r[Register::H] * 0x100 + r[Register::L]] + kequal::ite(carry, 1, 0)) % 0x100
                    == 0,
                n: false,
                h: r[Register::A] % 0x10 + m[r[Register::H] * 0x100 + r[Register::L]] % 0x10 + kequal::ite(carry, 1, 0)
                    >= 0x10,
                c: r[Register::A] + m[r[Register::H] * 0x100 + r[Register::L]] + kequal::ite(carry, 1, 0)
                    >= 0x100,
            },
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// SUB A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= r[src] && r[src] < 0x100),
        )]
        contract_alu_sub_register: {
            k: [Instruction::Alu8 { op: Alu::Sub, src: Operand::Reg { r: src } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            regs: k::map![Register::A -> (((r[Register::A] + 0x200 - r[src] - 0)) % 0x100), ..r],
            f: Flags {
                z: (r[Register::A] + 0x200 - r[src]) % 0x100 == 0,
                n: true,
                h: r[Register::A] % 0x10 < r[src] % 0x10,
                c: r[Register::A] < r[src],
            },
            z: r[src],
            bus: log,
        };

        /// SUB A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[pc] && m[pc] < 0x100)
                && (0 <= pc && pc < 0x1_0000),
        )]
        contract_alu_sub_immediate: {
            k: [Instruction::Alu8 { op: Alu::Sub, src: Operand::Immediate }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
            pc: pc,
        } => {
            k: [],
            regs: k::map![Register::A -> (((r[Register::A] + 0x200 - m[pc] - 0)) % 0x100), ..r],
            f: Flags {
                z: (r[Register::A] + 0x200 - m[pc]) % 0x100 == 0,
                n: true,
                h: r[Register::A] % 0x10 < m[pc] % 0x10,
                c: r[Register::A] < m[pc],
            },
            z: m[pc],
            bus: [..log, Event::Read { addr: pc, value: m[pc] }],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// SUB A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_alu_sub_indirect: {
            k: [Instruction::Alu8 { op: Alu::Sub, src: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            regs: k::map![
                Register::A -> (
                    (
                        (r[Register::A] + 0x200 - m[r[Register::H] * 0x100 + r[Register::L]] - 0)
                    ) % 0x100
                ),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] + 0x200 - m[r[Register::H] * 0x100 + r[Register::L]]) % 0x100 == 0,
                n: true,
                h: r[Register::A] % 0x10 < m[r[Register::H] * 0x100 + r[Register::L]] % 0x10,
                c: r[Register::A] < m[r[Register::H] * 0x100 + r[Register::L]],
            },
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// SBC A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= r[src] && r[src] < 0x100),
        )]
        contract_alu_sbc_register: {
            k: [Instruction::Alu8 { op: Alu::Sbc, src: Operand::Reg { r: src } }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            regs: k::map![
                Register::A -> (((r[Register::A] + 0x200 - r[src] - kequal::ite(carry, 1, 0))) % 0x100),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] + 0x200 - r[src] - kequal::ite(carry, 1, 0)) % 0x100 == 0,
                n: true,
                h: r[Register::A] % 0x10 < r[src] % 0x10 + kequal::ite(carry, 1, 0),
                c: r[Register::A] < r[src] + kequal::ite(carry, 1, 0),
            },
            z: r[src],
            bus: log,
        };

        /// SBC A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[pc] && m[pc] < 0x100)
                && (0 <= pc && pc < 0x1_0000),
        )]
        contract_alu_sbc_immediate: {
            k: [Instruction::Alu8 { op: Alu::Sbc, src: Operand::Immediate }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
            memory: m,
            pc: pc,
        } => {
            k: [],
            regs: k::map![
                Register::A -> (((r[Register::A] + 0x200 - m[pc] - kequal::ite(carry, 1, 0))) % 0x100),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] + 0x200 - m[pc] - kequal::ite(carry, 1, 0)) % 0x100 == 0,
                n: true,
                h: r[Register::A] % 0x10 < m[pc] % 0x10 + kequal::ite(carry, 1, 0),
                c: r[Register::A] < m[pc] + kequal::ite(carry, 1, 0),
            },
            z: m[pc],
            bus: [..log, Event::Read { addr: pc, value: m[pc] }],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// SBC A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_alu_sbc_indirect: {
            k: [Instruction::Alu8 { op: Alu::Sbc, src: Operand::HlIndirect }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            regs: k::map![
                Register::A -> (
                    (r[Register::A] + 0x200 - m[r[Register::H] * 0x100 + r[Register::L]] - kequal::ite(carry, 1, 0)) % 0x100
                ),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] + 0x200 - m[r[Register::H] * 0x100 + r[Register::L]] - kequal::ite(carry, 1, 0))
                    % 0x100
                    == 0,
                n: true,
                h: r[Register::A] % 0x10
                    < m[r[Register::H] * 0x100 + r[Register::L]] % 0x10 + kequal::ite(carry, 1, 0),
                c: r[Register::A] < m[r[Register::H] * 0x100 + r[Register::L]] + kequal::ite(carry, 1, 0),
            },
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// AND A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= r[src] && r[src] < 0x100),
        )]
        contract_alu_and_register: {
            k: [Instruction::Alu8 { op: Alu::And, src: Operand::Reg { r: src } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            regs: k::map![Register::A -> (r[Register::A] & r[src]), ..r],
            f: Flags { z: (r[Register::A] & r[src]) == 0, n: false, h: true, c: false },
            z: r[src],
            bus: log,
        };

        /// AND A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[pc] && m[pc] < 0x100)
                && (0 <= pc && pc < 0x1_0000),
        )]
        contract_alu_and_immediate: {
            k: [Instruction::Alu8 { op: Alu::And, src: Operand::Immediate }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
            pc: pc,
        } => {
            k: [],
            regs: k::map![Register::A -> (r[Register::A] & m[pc]), ..r],
            f: Flags { z: (r[Register::A] & m[pc]) == 0, n: false, h: true, c: false },
            z: m[pc],
            bus: [..log, Event::Read { addr: pc, value: m[pc] }],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// AND A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_alu_and_indirect: {
            k: [Instruction::Alu8 { op: Alu::And, src: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            regs: k::map![
                Register::A -> (r[Register::A] & m[r[Register::H] * 0x100 + r[Register::L]]),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] & m[r[Register::H] * 0x100 + r[Register::L]]) == 0,
                n: false,
                h: true,
                c: false,
            },
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// XOR A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= r[src] && r[src] < 0x100),
        )]
        contract_alu_xor_register: {
            k: [Instruction::Alu8 { op: Alu::Xor, src: Operand::Reg { r: src } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            regs: k::map![Register::A -> (r[Register::A] ^ r[src]), ..r],
            f: Flags { z: (r[Register::A] ^ r[src]) == 0, n: false, h: false, c: false },
            z: r[src],
            bus: log,
        };

        /// XOR A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[pc] && m[pc] < 0x100)
                && (0 <= pc && pc < 0x1_0000),
        )]
        contract_alu_xor_immediate: {
            k: [Instruction::Alu8 { op: Alu::Xor, src: Operand::Immediate }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
            pc: pc,
        } => {
            k: [],
            regs: k::map![Register::A -> (r[Register::A] ^ m[pc]), ..r],
            f: Flags { z: (r[Register::A] ^ m[pc]) == 0, n: false, h: false, c: false },
            z: m[pc],
            bus: [..log, Event::Read { addr: pc, value: m[pc] }],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// XOR A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_alu_xor_indirect: {
            k: [Instruction::Alu8 { op: Alu::Xor, src: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            regs: k::map![
                Register::A -> (r[Register::A] ^ m[r[Register::H] * 0x100 + r[Register::L]]),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] ^ m[r[Register::H] * 0x100 + r[Register::L]]) == 0,
                n: false,
                h: false,
                c: false,
            },
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// OR A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= r[src] && r[src] < 0x100),
        )]
        contract_alu_or_register: {
            k: [Instruction::Alu8 { op: Alu::Or, src: Operand::Reg { r: src } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            regs: k::map![Register::A -> (r[Register::A] | r[src]), ..r],
            f: Flags { z: (r[Register::A] | r[src]) == 0, n: false, h: false, c: false },
            z: r[src],
            bus: log,
        };

        /// OR A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[pc] && m[pc] < 0x100)
                && (0 <= pc && pc < 0x1_0000),
        )]
        contract_alu_or_immediate: {
            k: [Instruction::Alu8 { op: Alu::Or, src: Operand::Immediate }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
            pc: pc,
        } => {
            k: [],
            regs: k::map![Register::A -> (r[Register::A] | m[pc]), ..r],
            f: Flags { z: (r[Register::A] | m[pc]) == 0, n: false, h: false, c: false },
            z: m[pc],
            bus: [..log, Event::Read { addr: pc, value: m[pc] }],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// OR A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_alu_or_indirect: {
            k: [Instruction::Alu8 { op: Alu::Or, src: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            regs: k::map![
                Register::A -> (r[Register::A] | m[r[Register::H] * 0x100 + r[Register::L]]),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] | m[r[Register::H] * 0x100 + r[Register::L]]) == 0,
                n: false,
                h: false,
                c: false,
            },
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// CP A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= r[src] && r[src] < 0x100),
        )]
        contract_alu_cp_register: {
            k: [Instruction::Alu8 { op: Alu::Cp, src: Operand::Reg { r: src } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            regs: k::map![Register::A -> r[Register::A], ..r],
            f: Flags {
                z: (r[Register::A] + 0x200 - r[src]) % 0x100 == 0,
                n: true,
                h: r[Register::A] % 0x10 < r[src] % 0x10,
                c: r[Register::A] < r[src],
            },
            z: r[src],
            bus: log,
        };

        /// CP A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[pc] && m[pc] < 0x100)
                && (0 <= pc && pc < 0x1_0000),
        )]
        contract_alu_cp_immediate: {
            k: [Instruction::Alu8 { op: Alu::Cp, src: Operand::Immediate }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
            pc: pc,
        } => {
            k: [],
            regs: k::map![Register::A -> r[Register::A], ..r],
            f: Flags {
                z: (r[Register::A] + 0x200 - m[pc]) % 0x100 == 0,
                n: true,
                h: r[Register::A] % 0x10 < m[pc] % 0x10,
                c: r[Register::A] < m[pc],
            },
            z: m[pc],
            bus: [..log, Event::Read { addr: pc, value: m[pc] }],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// CP A,source: independent byte arithmetic/logic and all four flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100)
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_alu_cp_indirect: {
            k: [Instruction::Alu8 { op: Alu::Cp, src: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            regs: k::map![Register::A -> r[Register::A], ..r],
            f: Flags {
                z: (r[Register::A] + 0x200 - m[r[Register::H] * 0x100 + r[Register::L]]) % 0x100 == 0,
                n: true,
                h: r[Register::A] % 0x10 < m[r[Register::H] * 0x100 + r[Register::L]] % 0x10,
                c: r[Register::A] < m[r[Register::H] * 0x100 + r[Register::L]],
            },
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// INC: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[target] && r[target] < 0x100),
        )]
        contract_unary_inc_register: {
            k: [Instruction::Update8 { op: Unary::Inc, target: Operand::Reg { r: target } }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags {
                z: (r[target] + 1) % 0x100 == 0,
                n: false,
                h: r[target] % 0x10 + 1 >= 0x10,
                c: carry,
            },
            z: ((r[target] + 1) % 0x100),
            bus: log,
            regs: k::map![target -> ((r[target] + 1) % 0x100), ..r],
        };

        /// INC: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                && m[r[Register::H] * 0x100 + r[Register::L]] < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_unary_inc_indirect: {
            k: [Instruction::Update8 { op: Unary::Inc, target: Operand::HlIndirect }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: (m[r[Register::H] * 0x100 + r[Register::L]] + 1) % 0x100 == 0,
                n: false,
                h: m[r[Register::H] * 0x100 + r[Register::L]] % 0x10 + 1 >= 0x10,
                c: carry,
            },
            z: ((m[r[Register::H] * 0x100 + r[Register::L]] + 1) % 0x100),
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: ((m[r[Register::H] * 0x100 + r[Register::L]] + 1) % 0x100),
                },
            ],
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    (m[r[Register::H] * 0x100 + r[Register::L]] + 1) % 0x100
                ),
                ..m,
            ],
        };

        /// DEC: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[target] && r[target] < 0x100),
        )]
        contract_unary_dec_register: {
            k: [Instruction::Update8 { op: Unary::Dec, target: Operand::Reg { r: target } }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags {
                z: (r[target] + 0xFF) % 0x100 == 0,
                n: true,
                h: r[target] % 0x10 == 0,
                c: carry,
            },
            z: ((r[target] + 0xFF) % 0x100),
            bus: log,
            regs: k::map![target -> ((r[target] + 0xFF) % 0x100), ..r],
        };

        /// DEC: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                && m[r[Register::H] * 0x100 + r[Register::L]] < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_unary_dec_indirect: {
            k: [Instruction::Update8 { op: Unary::Dec, target: Operand::HlIndirect }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: (m[r[Register::H] * 0x100 + r[Register::L]] + 0xFF) % 0x100 == 0,
                n: true,
                h: m[r[Register::H] * 0x100 + r[Register::L]] % 0x10 == 0,
                c: carry,
            },
            z: ((m[r[Register::H] * 0x100 + r[Register::L]] + 0xFF) % 0x100),
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: ((m[r[Register::H] * 0x100 + r[Register::L]] + 0xFF) % 0x100),
                },
            ],
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    (m[r[Register::H] * 0x100 + r[Register::L]] + 0xFF) % 0x100
                ),
                ..m,
            ],
        };

        /// RLC: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[target] && r[target] < 0x100),
        )]
        contract_unary_rlc_register: {
            k: [Instruction::Update8 { op: Unary::Rlc, target: Operand::Reg { r: target } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags {
                z: (r[target] * 2) % 0x100 + r[target] / 0x80 == 0,
                n: false,
                h: false,
                c: r[target] / 0x80 == 1,
            },
            z: ((r[target] * 2) % 0x100 + r[target] / 0x80),
            bus: log,
            regs: k::map![target -> ((r[target] * 2) % 0x100 + r[target] / 0x80), ..r],
        };

        /// RLC: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                && m[r[Register::H] * 0x100 + r[Register::L]] < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_unary_rlc_indirect: {
            k: [Instruction::Update8 { op: Unary::Rlc, target: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: (m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100 + m[r[Register::H] * 0x100 + r[Register::L]] / 0x80
                    == 0,
                n: false,
                h: false,
                c: m[r[Register::H] * 0x100 + r[Register::L]] / 0x80 == 1,
            },
            z: (
                (m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100 + m[
                    r[Register::H] * 0x100 + r[Register::L]
                ] / 0x80
            ),
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write { addr: (r[Register::H] * 0x100 + r[Register::L]), value: (
                        (m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100 + m[
                            r[Register::H] * 0x100 + r[Register::L]
                        ] / 0x80
                    ),
                },
            ],
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    (m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100 + m[
                        r[Register::H] * 0x100 + r[Register::L]
                    ] / 0x80
                ),
                ..m,
            ],
        };

        /// RRC: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[target] && r[target] < 0x100),
        )]
        contract_unary_rrc_register: {
            k: [Instruction::Update8 { op: Unary::Rrc, target: Operand::Reg { r: target } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags {
                z: r[target] / 2 + (r[target] % 2) * 0x80 == 0,
                n: false,
                h: false,
                c: r[target] % 2 == 1,
            },
            z: (r[target] / 2 + (r[target] % 2) * 0x80),
            bus: log,
            regs: k::map![target -> (r[target] / 2 + (r[target] % 2) * 0x80), ..r],
        };

        /// RRC: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                && m[r[Register::H] * 0x100 + r[Register::L]] < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_unary_rrc_indirect: {
            k: [Instruction::Update8 { op: Unary::Rrc, target: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: m[r[Register::H] * 0x100 + r[Register::L]] / 2 + (m[r[Register::H] * 0x100 + r[Register::L]] % 2) * 0x80
                    == 0,
                n: false,
                h: false,
                c: m[r[Register::H] * 0x100 + r[Register::L]] % 2 == 1,
            },
            z: (
                m[r[Register::H] * 0x100 + r[Register::L]] / 2 + (
                    m[r[Register::H] * 0x100 + r[Register::L]] % 2
                ) * 0x80
            ),
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write { addr: (r[Register::H] * 0x100 + r[Register::L]), value: (
                        m[r[Register::H] * 0x100 + r[Register::L]] / 2 + (
                            m[r[Register::H] * 0x100 + r[Register::L]] % 2
                        ) * 0x80
                    ),
                },
            ],
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    m[r[Register::H] * 0x100 + r[Register::L]] / 2 + (
                        m[r[Register::H] * 0x100 + r[Register::L]] % 2
                    ) * 0x80
                ),
                ..m,
            ],
        };

        /// RL: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[target] && r[target] < 0x100),
        )]
        contract_unary_rl_register: {
            k: [Instruction::Update8 { op: Unary::Rl, target: Operand::Reg { r: target } }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags {
                z: (r[target] * 2) % 0x100 + kequal::ite(carry, 1, 0) == 0,
                n: false,
                h: false,
                c: r[target] / 0x80 == 1,
            },
            z: ((r[target] * 2) % 0x100 + kequal::ite(carry, 1, 0)),
            bus: log,
            regs: k::map![target -> ((r[target] * 2) % 0x100 + kequal::ite(carry, 1, 0)), ..r],
        };

        /// RL: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                && m[r[Register::H] * 0x100 + r[Register::L]] < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_unary_rl_indirect: {
            k: [Instruction::Update8 { op: Unary::Rl, target: Operand::HlIndirect }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: (m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100 + kequal::ite(carry, 1, 0) == 0,
                n: false,
                h: false,
                c: m[r[Register::H] * 0x100 + r[Register::L]] / 0x80 == 1,
            },
            z: ((m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100 + kequal::ite(carry, 1, 0)),
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write { addr: (r[Register::H] * 0x100 + r[Register::L]), value: (
                        (m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100 + kequal::ite(carry, 1, 0)
                    ),
                },
            ],
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    (m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100 + kequal::ite(carry, 1, 0)
                ),
                ..m,
            ],
        };

        /// RR: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[target] && r[target] < 0x100),
        )]
        contract_unary_rr_register: {
            k: [Instruction::Update8 { op: Unary::Rr, target: Operand::Reg { r: target } }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags {
                z: r[target] / 2 + kequal::ite(carry, 1, 0) * 0x80 == 0,
                n: false,
                h: false,
                c: r[target] % 2 == 1,
            },
            z: (r[target] / 2 + kequal::ite(carry, 1, 0) * 0x80),
            bus: log,
            regs: k::map![target -> (r[target] / 2 + kequal::ite(carry, 1, 0) * 0x80), ..r],
        };

        /// RR: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                && m[r[Register::H] * 0x100 + r[Register::L]] < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_unary_rr_indirect: {
            k: [Instruction::Update8 { op: Unary::Rr, target: Operand::HlIndirect }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: m[r[Register::H] * 0x100 + r[Register::L]] / 2 + kequal::ite(carry, 1, 0) * 0x80 == 0,
                n: false,
                h: false,
                c: m[r[Register::H] * 0x100 + r[Register::L]] % 2 == 1,
            },
            z: (m[r[Register::H] * 0x100 + r[Register::L]] / 2 + kequal::ite(carry, 1, 0) * 0x80),
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write { addr: (r[Register::H] * 0x100 + r[Register::L]), value: (
                        m[r[Register::H] * 0x100 + r[Register::L]] / 2 + kequal::ite(carry, 1, 0) * 0x80
                    ),
                },
            ],
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    m[r[Register::H] * 0x100 + r[Register::L]] / 2 + kequal::ite(carry, 1, 0) * 0x80
                ),
                ..m,
            ],
        };

        /// SLA: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[target] && r[target] < 0x100),
        )]
        contract_unary_sla_register: {
            k: [Instruction::Update8 { op: Unary::Sla, target: Operand::Reg { r: target } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: (r[target] * 2) % 0x100 == 0, n: false, h: false, c: r[target] / 0x80 == 1 },
            z: ((r[target] * 2) % 0x100),
            bus: log,
            regs: k::map![target -> ((r[target] * 2) % 0x100), ..r],
        };

        /// SLA: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                && m[r[Register::H] * 0x100 + r[Register::L]] < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_unary_sla_indirect: {
            k: [Instruction::Update8 { op: Unary::Sla, target: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: (m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100 == 0,
                n: false,
                h: false,
                c: m[r[Register::H] * 0x100 + r[Register::L]] / 0x80 == 1,
            },
            z: ((m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100),
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: ((m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100),
                },
            ],
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    (m[r[Register::H] * 0x100 + r[Register::L]] * 2) % 0x100
                ),
                ..m,
            ],
        };

        /// SRA: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[target] && r[target] < 0x100),
        )]
        contract_unary_sra_register: {
            k: [Instruction::Update8 { op: Unary::Sra, target: Operand::Reg { r: target } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags {
                z: r[target] / 2 + (r[target] / 0x80) * 0x80 == 0,
                n: false,
                h: false,
                c: r[target] % 2 == 1,
            },
            z: (r[target] / 2 + (r[target] / 0x80) * 0x80),
            bus: log,
            regs: k::map![target -> (r[target] / 2 + (r[target] / 0x80) * 0x80), ..r],
        };

        /// SRA: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                && m[r[Register::H] * 0x100 + r[Register::L]] < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_unary_sra_indirect: {
            k: [Instruction::Update8 { op: Unary::Sra, target: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: m[r[Register::H] * 0x100 + r[Register::L]] / 2 + (m[r[Register::H] * 0x100 + r[Register::L]] / 0x80) * 0x80
                    == 0,
                n: false,
                h: false,
                c: m[r[Register::H] * 0x100 + r[Register::L]] % 2 == 1,
            },
            z: (
                m[r[Register::H] * 0x100 + r[Register::L]] / 2 + (
                    m[r[Register::H] * 0x100 + r[Register::L]] / 0x80
                ) * 0x80
            ),
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write { addr: (r[Register::H] * 0x100 + r[Register::L]), value: (
                        m[r[Register::H] * 0x100 + r[Register::L]] / 2 + (
                            m[r[Register::H] * 0x100 + r[Register::L]] / 0x80
                        ) * 0x80
                    ),
                },
            ],
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    m[r[Register::H] * 0x100 + r[Register::L]] / 2 + (
                        m[r[Register::H] * 0x100 + r[Register::L]] / 0x80
                    ) * 0x80
                ),
                ..m,
            ],
        };

        /// SWAP: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[target] && r[target] < 0x100),
        )]
        contract_unary_swap_register: {
            k: [Instruction::Update8 { op: Unary::Swap, target: Operand::Reg { r: target } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: r[target] == 0, n: false, h: false, c: false },
            z: ((r[target] % 0x10) * 0x10 + r[target] / 0x10),
            bus: log,
            regs: k::map![target -> ((r[target] % 0x10) * 0x10 + r[target] / 0x10), ..r],
        };

        /// SWAP: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                && m[r[Register::H] * 0x100 + r[Register::L]] < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_unary_swap_indirect: {
            k: [Instruction::Update8 { op: Unary::Swap, target: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: m[r[Register::H] * 0x100 + r[Register::L]] == 0,
                n: false,
                h: false,
                c: false,
            },
            z: (
                (m[r[Register::H] * 0x100 + r[Register::L]] % 0x10) * 0x10 + m[
                    r[Register::H] * 0x100 + r[Register::L]
                ] / 0x10
            ),
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write { addr: (r[Register::H] * 0x100 + r[Register::L]), value: (
                        (m[r[Register::H] * 0x100 + r[Register::L]] % 0x10) * 0x10 + m[
                            r[Register::H] * 0x100 + r[Register::L]
                        ] / 0x10
                    ),
                },
            ],
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    (m[r[Register::H] * 0x100 + r[Register::L]] % 0x10) * 0x10 + m[
                        r[Register::H] * 0x100 + r[Register::L]
                    ] / 0x10
                ),
                ..m,
            ],
        };

        /// SRL: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[target] && r[target] < 0x100),
        )]
        contract_unary_srl_register: {
            k: [Instruction::Update8 { op: Unary::Srl, target: Operand::Reg { r: target } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: r[target] / 2 == 0, n: false, h: false, c: r[target] % 2 == 1 },
            z: (r[target] / 2),
            bus: log,
            regs: k::map![target -> (r[target] / 2), ..r],
        };

        /// SRL: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                && m[r[Register::H] * 0x100 + r[Register::L]] < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_unary_srl_indirect: {
            k: [Instruction::Update8 { op: Unary::Srl, target: Operand::HlIndirect }],
            regs: r,
            f: _,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: m[r[Register::H] * 0x100 + r[Register::L]] / 2 == 0,
                n: false,
                h: false,
                c: m[r[Register::H] * 0x100 + r[Register::L]] % 2 == 1,
            },
            z: (m[r[Register::H] * 0x100 + r[Register::L]] / 2),
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: (m[r[Register::H] * 0x100 + r[Register::L]] / 2),
                },
            ],
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    m[r[Register::H] * 0x100 + r[Register::L]] / 2
                ),
                ..m,
            ],
        };

        /// RLCA: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100),
        )]
        contract_unary_rlca_accumulator: {
            k: [Instruction::Update8 { op: Unary::Rlca, target: Operand::Reg { r: Register::A } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: false, n: false, h: false, c: r[Register::A] / 0x80 == 1 },
            z: ((r[Register::A] * 2) % 0x100 + r[Register::A] / 0x80),
            bus: log,
            regs: k::map![
                Register::A -> ((r[Register::A] * 2) % 0x100 + r[Register::A] / 0x80),
                ..r,
            ],
        };

        /// RRCA: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100),
        )]
        contract_unary_rrca_accumulator: {
            k: [Instruction::Update8 { op: Unary::Rrca, target: Operand::Reg { r: Register::A } }],
            regs: r,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: false, n: false, h: false, c: r[Register::A] % 2 == 1 },
            z: (r[Register::A] / 2 + (r[Register::A] % 2) * 0x80),
            bus: log,
            regs: k::map![Register::A -> (r[Register::A] / 2 + (r[Register::A] % 2) * 0x80), ..r],
        };

        /// RLA: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100),
        )]
        contract_unary_rla_accumulator: {
            k: [Instruction::Update8 { op: Unary::Rla, target: Operand::Reg { r: Register::A } }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: false, n: false, h: false, c: r[Register::A] / 0x80 == 1 },
            z: ((r[Register::A] * 2) % 0x100 + kequal::ite(carry, 1, 0)),
            bus: log,
            regs: k::map![
                Register::A -> ((r[Register::A] * 2) % 0x100 + kequal::ite(carry, 1, 0)),
                ..r,
            ],
        };

        /// RRA: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100),
        )]
        contract_unary_rra_accumulator: {
            k: [Instruction::Update8 { op: Unary::Rra, target: Operand::Reg { r: Register::A } }],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: false, n: false, h: false, c: r[Register::A] % 2 == 1 },
            z: (r[Register::A] / 2 + kequal::ite(carry, 1, 0) * 0x80),
            bus: log,
            regs: k::map![Register::A -> (r[Register::A] / 2 + kequal::ite(carry, 1, 0) * 0x80), ..r],
        };

        /// DAA: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100),
        )]
        contract_unary_daa_accumulator: {
            k: [Instruction::Update8 { op: Unary::Daa, target: Operand::Reg { r: Register::A } }],
            regs: r,
            f: Flags { n: subtract, h: half_carry, c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags {
                z: ExpectedDaaByte(r[Register::A], subtract, half_carry, carry) == 0,
                n: subtract,
                h: false,
                c: ExpectedDaaHigh(r[Register::A], subtract, carry) != 0,
            },
            z: ExpectedDaaByte(r[Register::A], subtract, half_carry, carry),
            bus: log,
            regs: k::map![
                Register::A -> ExpectedDaaByte(r[Register::A], subtract, half_carry, carry),
                ..r,
            ],
        };

        /// CPL: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100),
        )]
        contract_unary_cpl_accumulator: {
            k: [Instruction::Update8 { op: Unary::Cpl, target: Operand::Reg { r: Register::A } }],
            regs: r,
            f: Flags { z: zero, c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: zero, n: true, h: true, c: carry },
            z: (0xFF - r[Register::A]),
            bus: log,
            regs: k::map![Register::A -> (0xFF - r[Register::A]), ..r],
        };

        /// SCF: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100),
        )]
        contract_unary_scf_accumulator: {
            k: [Instruction::Update8 { op: Unary::Scf, target: Operand::Reg { r: Register::A } }],
            regs: r,
            f: Flags { z: zero, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: zero, n: false, h: false, c: true },
            z: r[Register::A],
            bus: log,
            regs: k::map![Register::A -> r[Register::A], ..r],
        };

        /// CCF: byte result, flag behavior and read/modify/write ordering where applicable.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::A] && r[Register::A] < 0x100),
        )]
        contract_unary_ccf_accumulator: {
            k: [Instruction::Update8 { op: Unary::Ccf, target: Operand::Reg { r: Register::A } }],
            regs: r,
            f: Flags { z: zero, c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: zero, n: false, h: false, c: !carry },
            z: r[Register::A],
            bus: log,
            regs: k::map![Register::A -> r[Register::A], ..r],
        };

        /// ResetBit: all bit positions 0..7, preserving required flags and unrelated state.
        #[k::claim(all_paths)]
        #[k::requires(
            0 <= bit
                && bit < 8
                && (0 <= r[target] && r[target] < 0x100),
        )]
        contract_bit_resetbit_register: {
            k: [
                Instruction::Update8 {
                    op: Unary::ResetBit { bit: bit @ (0 | 1 | 2 | 3 | 4 | 5 | 6 | 7) },
                    target: Operand::Reg { r: target },
                },
            ],
            regs: r,
            z: _,
            bus: log,
        } => {
            k: [],
            z: (r[target] - ((r[target] / (1 << bit)) % 2) * (1 << bit)),
            regs: k::map![target -> (r[target] - ((r[target] / (1 << bit)) % 2) * (1 << bit)), ..r],
            bus: log,
        };

        /// ResetBit: all bit positions 0..7, preserving required flags and unrelated state.
        #[k::claim(all_paths)]
        #[k::requires(
            0 <= bit
                && bit < 8
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_bit_resetbit_indirect: {
            k: [
                Instruction::Update8 {
                    op: Unary::ResetBit { bit: bit @ (0 | 1 | 2 | 3 | 4 | 5 | 6 | 7) },
                    target: Operand::HlIndirect,
                },
            ],
            regs: r,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            z: (
                m[r[Register::H] * 0x100 + r[Register::L]] - (
                    (m[r[Register::H] * 0x100 + r[Register::L]] / (1 << bit)) % 2
                ) * (1 << bit)
            ),
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    m[r[Register::H] * 0x100 + r[Register::L]] - (
                        (m[r[Register::H] * 0x100 + r[Register::L]] / (1 << bit)) % 2
                    ) * (1 << bit)
                ),
                ..m,
            ],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write { addr: (r[Register::H] * 0x100 + r[Register::L]), value: (
                        m[r[Register::H] * 0x100 + r[Register::L]] - (
                            (m[r[Register::H] * 0x100 + r[Register::L]] / (1 << bit)) % 2
                        ) * (1 << bit)
                    ),
                },
            ],
        };

        /// SetBit: all bit positions 0..7, preserving required flags and unrelated state.
        #[k::claim(all_paths)]
        #[k::requires(
            0 <= bit
                && bit < 8
                && (0 <= r[target] && r[target] < 0x100),
        )]
        contract_bit_setbit_register: {
            k: [
                Instruction::Update8 {
                    op: Unary::SetBit { bit: bit @ (0 | 1 | 2 | 3 | 4 | 5 | 6 | 7) },
                    target: Operand::Reg { r: target },
                },
            ],
            regs: r,
            z: _,
            bus: log,
        } => {
            k: [],
            z: (r[target] - ((r[target] / (1 << bit)) % 2) * (1 << bit) + (1 << bit)),
            regs: k::map![
                target -> (r[target] - ((r[target] / (1 << bit)) % 2) * (1 << bit) + (1 << bit)),
                ..r,
            ],
            bus: log,
        };

        /// SetBit: all bit positions 0..7, preserving required flags and unrelated state.
        #[k::claim(all_paths)]
        #[k::requires(
            0 <= bit
                && bit < 8
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_bit_setbit_indirect: {
            k: [
                Instruction::Update8 {
                    op: Unary::SetBit { bit: bit @ (0 | 1 | 2 | 3 | 4 | 5 | 6 | 7) },
                    target: Operand::HlIndirect,
                },
            ],
            regs: r,
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            z: (
                m[r[Register::H] * 0x100 + r[Register::L]] - (
                    (m[r[Register::H] * 0x100 + r[Register::L]] / (1 << bit)) % 2
                ) * (1 << bit) + (1 << bit)
            ),
            memory: k::map![
                (r[Register::H] * 0x100 + r[Register::L]) -> (
                    m[r[Register::H] * 0x100 + r[Register::L]] - (
                        (m[r[Register::H] * 0x100 + r[Register::L]] / (1 << bit)) % 2
                    ) * (1 << bit) + (1 << bit)
                ),
                ..m,
            ],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
                Event::Write { addr: (r[Register::H] * 0x100 + r[Register::L]), value: (
                        m[r[Register::H] * 0x100 + r[Register::L]] - (
                            (m[r[Register::H] * 0x100 + r[Register::L]] / (1 << bit)) % 2
                        ) * (1 << bit) + (1 << bit)
                    ),
                },
            ],
        };

        /// TestBit: all bit positions 0..7, preserving required flags and unrelated state.
        #[k::claim(all_paths)]
        #[k::requires(
            0 <= bit
                && bit < 8
                && (0 <= r[target] && r[target] < 0x100),
        )]
        contract_bit_testbit_register: {
            k: [
                Instruction::TestBit {
                    bit: bit @ (0 | 1 | 2 | 3 | 4 | 5 | 6 | 7),
                    target: Operand::Reg { r: target },
                },
            ],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
        } => {
            k: [],
            f: Flags { z: (r[target] / (1 << bit)) % 2 == 0, n: false, h: true, c: carry },
            z: r[target],
            bus: log,
        };

        /// TestBit: all bit positions 0..7, preserving required flags and unrelated state.
        #[k::claim(all_paths)]
        #[k::requires(
            0 <= bit
                && bit < 8
                && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                    && m[r[Register::H] * 0x100 + r[Register::L]]
                        < 0x100)
                && (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_bit_testbit_indirect: {
            k: [
                Instruction::TestBit {
                    bit: bit @ (0 | 1 | 2 | 3 | 4 | 5 | 6 | 7),
                    target: Operand::HlIndirect,
                },
            ],
            regs: r,
            f: Flags { c: carry, .. },
            z: _,
            bus: log,
            memory: m,
        } => {
            k: [],
            f: Flags {
                z: (m[r[Register::H] * 0x100 + r[Register::L]] / (1 << bit)) % 2 == 0,
                n: false,
                h: true,
                c: carry,
            },
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// 16-bit increment/decrement wraps, preserves flags and takes one internal cycle.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[high] && r[high] < 0x100)
                && (0 <= r[low] && r[low] < 0x100),
        )]
        contract_inc16_registers: {
            k: [Instruction::Inc16 { pair: Pair16::Registers { high, low } }],
            regs: r,
            sp: sp,
            bus: log,
        } => {
            k: [],
            bus: [..log, Event::Idle],
            regs: k::map![
                high -> (((r[high] * 0x100 + r[low]) + 1) % 0x1_0000) / 0x100,
                low -> (((r[high] * 0x100 + r[low]) + 1) % 0x1_0000) % 0x100,
                ..r,
            ],
        };

        /// 16-bit increment/decrement wraps, preserves flags and takes one internal cycle.
        #[k::claim(all_paths)]
        #[k::requires((0 <= sp && sp < 0x1_0000))]
        contract_inc16_sp: {
            k: [Instruction::Inc16 { pair: Pair16::StackPointer }],
            regs: r,
            sp: sp,
            bus: log,
        } => {
            k: [],
            bus: [..log, Event::Idle],
            sp: ((sp + 1) % 0x1_0000),
        };

        /// 16-bit increment/decrement wraps, preserves flags and takes one internal cycle.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[high] && r[high] < 0x100)
                && (0 <= r[low] && r[low] < 0x100),
        )]
        contract_dec16_registers: {
            k: [Instruction::Dec16 { pair: Pair16::Registers { high, low } }],
            regs: r,
            sp: sp,
            bus: log,
        } => {
            k: [],
            bus: [..log, Event::Idle],
            regs: k::map![
                high -> (((r[high] * 0x100 + r[low]) + 0xFFFF) % 0x1_0000) / 0x100,
                low -> (((r[high] * 0x100 + r[low]) + 0xFFFF) % 0x1_0000) % 0x100,
                ..r,
            ],
        };

        /// 16-bit increment/decrement wraps, preserves flags and takes one internal cycle.
        #[k::claim(all_paths)]
        #[k::requires((0 <= sp && sp < 0x1_0000))]
        contract_dec16_sp: {
            k: [Instruction::Dec16 { pair: Pair16::StackPointer }],
            regs: r,
            sp: sp,
            bus: log,
        } => {
            k: [],
            bus: [..log, Event::Idle],
            sp: ((sp + 0xFFFF) % 0x1_0000),
        };

        /// JP HL: jump to the register pair, with no extra bus cycle.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100),
        )]
        contract_jump_hl: {
            k: [Instruction::JumpHl],
            regs: r,
            pc: _,
        } => {
            k: [],
            pc: (r[Register::H] * 0x100 + r[Register::L]),
        };

        /// DI: disable interrupts immediately and cancel pending EI.
        #[k::claim(all_paths)]
        contract_di: {
            k: [Instruction::DisableInterrupts],
            ime: _,
            ei_pending: _,
        } => {
            k: [],
            ime: false,
            ei_pending: false,
        };

        /// EI: schedule delayed enable; do not immediately change IME.
        #[k::claim(all_paths)]
        contract_ei: {
            k: [Instruction::EnableInterrupts],
            ei_pending: _,
        } => {
            k: [],
            ei_pending: true,
        };

        /// ADD HL,r16: wrap, carry out of bits 11 and 15, preserve Z, clear N; includes source=HL.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100)
                && (0 <= r[high] && r[high] < 0x100)
                && (0 <= r[low] && r[low] < 0x100),
        )]
        contract_add_hl_registers: {
            k: [Instruction::AddHl { pair: Pair16::Registers { high, low } }],
            regs: r,
            f: Flags { z: zero, .. },
            bus: log,
        } => {
            k: [],
            regs: k::map![
                Register::H -> (
                    (
                        ((r[Register::H] * 0x100 + r[Register::L]) + (r[high] * 0x100 + r[low]))
                    ) % 0x1_0000
                ) / 0x100,
                Register::L -> (
                    (
                        ((r[Register::H] * 0x100 + r[Register::L]) + (r[high] * 0x100 + r[low]))
                    ) % 0x1_0000
                ) % 0x100,
                ..r,
            ],
            f: Flags {
                z: zero,
                n: false,
                h: (r[Register::H] * 0x100 + r[Register::L]) % 0x1000 + (r[high] * 0x100 + r[low]) % 0x1000
                    >= 0x1000,
                c: (r[Register::H] * 0x100 + r[Register::L]) + (r[high] * 0x100 + r[low]) >= 0x1_0000,
            },
            bus: [..log, Event::Idle],
        };

        /// ADD HL,r16: wrap, carry out of bits 11 and 15, preserve Z, clear N; includes source=HL.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= r[Register::H] && r[Register::H] < 0x100)
                && (0 <= r[Register::L] && r[Register::L] < 0x100)
                && (0 <= sp && sp < 0x1_0000),
        )]
        contract_add_hl_sp: {
            k: [Instruction::AddHl { pair: Pair16::StackPointer }],
            regs: r,
            f: Flags { z: zero, .. },
            bus: log,
            sp: sp,
        } => {
            k: [],
            regs: k::map![
                Register::H -> (
                    (((r[Register::H] * 0x100 + r[Register::L]) + sp)) % 0x1_0000
                ) / 0x100,
                Register::L -> (
                    (((r[Register::H] * 0x100 + r[Register::L]) + sp)) % 0x1_0000
                ) % 0x100,
                ..r,
            ],
            f: Flags {
                z: zero,
                n: false,
                h: (r[Register::H] * 0x100 + r[Register::L]) % 0x1000 + sp % 0x1000 >= 0x1000,
                c: (r[Register::H] * 0x100 + r[Register::L]) + sp >= 0x1_0000,
            },
            bus: [..log, Event::Idle],
        };
    }
}
