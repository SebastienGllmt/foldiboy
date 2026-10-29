//! Branch, call, return and restart contracts, after opcode fetch.
//! Behavior: RGBDS gbz80(7), https://rgbds.gbdev.io/docs/gbz80.7 .
//! Encoding/families: Pan Docs CPU Instruction Set; bus order: izik1/gbops dmgops.json.
//! See README.md for pinned sources, coverage and model exclusions.
//! These are authored proof inputs; none calls the implementation's ALU/result helpers.

use k::builtin::{int, kequal};
use crate::decode::{Address, Condition, Instruction, Operand, Pair16};
use crate::alu::{Alu, Unary};
use crate::machine::{Cycle, Register};
use crate::Machine;

#[k::spec_module(name = "SM83-CONTROL-SPEC", imports = [crate])]
pub mod spec {
    use super::*;
    k::claims! {
        for Machine;

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires((0 <= pc && pc < 0x1_0000) && (0 <= m[pc] && m[pc] < 0x100) && (0 <= flags && flags < 0x100 && flags % 0x10 == 0) && (kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && flags / 0x80 == 1) || (kequal::eq(cond, Condition::NotZero) && flags / 0x80 == 0) || (kequal::eq(cond, Condition::Carry) && (flags / 0x10) % 2 == 1) || (kequal::eq(cond, Condition::NotCarry) && (flags / 0x10) % 2 == 0)))]
        contract_jumprelative_taken: {
            k: [Instruction::JumpRelative { cond: cond @ (Condition::Always | Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: flags,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            z: m[pc],
            pc: int::rem_euclid(pc + 1 + kequal::ite(m[pc] < 0x80, m[pc], m[pc] - 0x100), 0x1_0000),
            bus: [..log, Cycle::Read { addr: pc, value: m[pc] }, Cycle::Internal],
        };

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires((0 <= pc && pc < 0x1_0000) && (0 <= m[pc] && m[pc] < 0x100) && (0 <= flags && flags < 0x100 && flags % 0x10 == 0) && !(kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && flags / 0x80 == 1) || (kequal::eq(cond, Condition::NotZero) && flags / 0x80 == 0) || (kequal::eq(cond, Condition::Carry) && (flags / 0x10) % 2 == 1) || (kequal::eq(cond, Condition::NotCarry) && (flags / 0x10) % 2 == 0)))]
        contract_jumprelative_not_taken: {
            k: [Instruction::JumpRelative { cond: cond @ (Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: flags,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            z: m[pc],
            pc: ((pc + 1) % 0x1_0000),
            bus: [..log, Cycle::Read { addr: pc, value: m[pc] }],
        };

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires((0 <= pc && pc < 0x1_0000) && (0 <= m[pc] && m[pc] < 0x100) && (0 <= flags && flags < 0x100 && flags % 0x10 == 0) && (kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && flags / 0x80 == 1) || (kequal::eq(cond, Condition::NotZero) && flags / 0x80 == 0) || (kequal::eq(cond, Condition::Carry) && (flags / 0x10) % 2 == 1) || (kequal::eq(cond, Condition::NotCarry) && (flags / 0x10) % 2 == 0)) && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100))]
        contract_jump_taken: {
            k: [Instruction::Jump { cond: cond @ (Condition::Always | Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: flags,
            memory: m,
            z: _,
            bus: log,
            w: _,
        } => {
            k: [],
            z: m[pc],
            w: m[(pc + 1) % 0x1_0000],
            pc: (m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]),
            bus: [..log, Cycle::Read { addr: pc, value: m[pc] }, Cycle::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] }, Cycle::Internal],
        };

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires((0 <= pc && pc < 0x1_0000) && (0 <= m[pc] && m[pc] < 0x100) && (0 <= flags && flags < 0x100 && flags % 0x10 == 0) && !(kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && flags / 0x80 == 1) || (kequal::eq(cond, Condition::NotZero) && flags / 0x80 == 0) || (kequal::eq(cond, Condition::Carry) && (flags / 0x10) % 2 == 1) || (kequal::eq(cond, Condition::NotCarry) && (flags / 0x10) % 2 == 0)) && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100))]
        contract_jump_not_taken: {
            k: [Instruction::Jump { cond: cond @ (Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: flags,
            memory: m,
            z: _,
            bus: log,
            w: _,
        } => {
            k: [],
            z: m[pc],
            w: m[(pc + 1) % 0x1_0000],
            pc: ((pc + 2) % 0x1_0000),
            bus: [..log, Cycle::Read { addr: pc, value: m[pc] }, Cycle::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] }],
        };

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires((0 <= pc && pc < 0x1_0000) && (0 <= m[pc] && m[pc] < 0x100) && (0 <= flags && flags < 0x100 && flags % 0x10 == 0) && (kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && flags / 0x80 == 1) || (kequal::eq(cond, Condition::NotZero) && flags / 0x80 == 0) || (kequal::eq(cond, Condition::Carry) && (flags / 0x10) % 2 == 1) || (kequal::eq(cond, Condition::NotCarry) && (flags / 0x10) % 2 == 0)) && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100) && (0 <= sp && sp < 0x1_0000) && m.contains_key((sp + 0xFFFF) % 0x1_0000) && m.contains_key((sp + 0xFFFE) % 0x1_0000) && (0 <= m[(sp + 0xFFFF) % 0x1_0000] && m[(sp + 0xFFFF) % 0x1_0000] < 0x100) && (0 <= m[(sp + 0xFFFE) % 0x1_0000] && m[(sp + 0xFFFE) % 0x1_0000] < 0x100))]
        contract_call_taken: {
            k: [Instruction::Call { cond: cond @ (Condition::Always | Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: flags,
            memory: m,
            z: _,
            bus: log,
            w: _,
            sp: sp,
        } => {
            k: [],
            z: m[pc],
            w: m[(pc + 1) % 0x1_0000],
            pc: (m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]),
            sp: ((sp + 0xFFFE) % 0x1_0000),
            memory: k::map![
                ((sp + 0xFFFF) % 0x1_0000) -> ((pc + 2) % 0x1_0000) / 0x100,
                ((sp + 0xFFFE) % 0x1_0000) -> ((pc + 2) % 0x1_0000) % 0x100,
                ..m,
            ],
            bus: [..log, Cycle::Read { addr: pc, value: m[pc] }, Cycle::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] }, Cycle::Internal, Cycle::Write { addr: ((sp + 0xFFFF) % 0x1_0000), old: m[(sp + 0xFFFF) % 0x1_0000], new: ((pc + 2) % 0x1_0000) / 0x100 }, Cycle::Write { addr: ((sp + 0xFFFE) % 0x1_0000), old: m[(sp + 0xFFFE) % 0x1_0000], new: ((pc + 2) % 0x1_0000) % 0x100 }],
        };

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires((0 <= pc && pc < 0x1_0000) && (0 <= m[pc] && m[pc] < 0x100) && (0 <= flags && flags < 0x100 && flags % 0x10 == 0) && !(kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && flags / 0x80 == 1) || (kequal::eq(cond, Condition::NotZero) && flags / 0x80 == 0) || (kequal::eq(cond, Condition::Carry) && (flags / 0x10) % 2 == 1) || (kequal::eq(cond, Condition::NotCarry) && (flags / 0x10) % 2 == 0)) && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100))]
        contract_call_not_taken: {
            k: [Instruction::Call { cond: cond @ (Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: flags,
            memory: m,
            z: _,
            bus: log,
            w: _,
        } => {
            k: [],
            z: m[pc],
            w: m[(pc + 1) % 0x1_0000],
            pc: ((pc + 2) % 0x1_0000),
            bus: [..log, Cycle::Read { addr: pc, value: m[pc] }, Cycle::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] }],
        };

        /// RET/RETI: pop low then high, final internal cycle; RETI enables IME immediately.
        #[k::claim(all_paths)]
        #[k::requires((0 <= sp && sp < 0x1_0000) && (0 <= m[sp] && m[sp] < 0x100) && (0 <= m[(sp + 1) % 0x1_0000] && m[(sp + 1) % 0x1_0000] < 0x100))]
        contract_return_taken: {
            k: [Instruction::Return],
            sp: sp,
            pc: _,
            f: flags,
            memory: m,
            w: _,
            z: _,
            bus: log,
        } => {
            k: [],
            sp: ((sp + 2) % 0x1_0000),
            pc: (m[(sp + 1) % 0x1_0000] * 0x100 + m[sp]),
            w: m[(sp + 1) % 0x1_0000],
            z: m[sp],
            bus: [..log, Cycle::Read { addr: sp, value: m[sp] }, Cycle::Read { addr: ((sp + 1) % 0x1_0000), value: m[(sp + 1) % 0x1_0000] }, Cycle::Internal],
        };

        /// RET/RETI: pop low then high, final internal cycle; RETI enables IME immediately.
        #[k::claim(all_paths)]
        #[k::requires((0 <= sp && sp < 0x1_0000) && (0 <= m[sp] && m[sp] < 0x100) && (0 <= m[(sp + 1) % 0x1_0000] && m[(sp + 1) % 0x1_0000] < 0x100) && (0 <= flags && flags < 0x100 && flags % 0x10 == 0) && (kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && flags / 0x80 == 1) || (kequal::eq(cond, Condition::NotZero) && flags / 0x80 == 0) || (kequal::eq(cond, Condition::Carry) && (flags / 0x10) % 2 == 1) || (kequal::eq(cond, Condition::NotCarry) && (flags / 0x10) % 2 == 0)))]
        contract_returnif_taken: {
            k: [Instruction::ReturnIf { cond: cond @ (Condition::Always | Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            sp: sp,
            pc: _,
            f: flags,
            memory: m,
            w: _,
            z: _,
            bus: log,
        } => {
            k: [],
            sp: ((sp + 2) % 0x1_0000),
            pc: (m[(sp + 1) % 0x1_0000] * 0x100 + m[sp]),
            w: m[(sp + 1) % 0x1_0000],
            z: m[sp],
            bus: [..log, Cycle::Internal, Cycle::Read { addr: sp, value: m[sp] }, Cycle::Read { addr: ((sp + 1) % 0x1_0000), value: m[(sp + 1) % 0x1_0000] }, Cycle::Internal],
        };

        /// RET/RETI: pop low then high, final internal cycle; RETI enables IME immediately.
        #[k::claim(all_paths)]
        #[k::requires((0 <= sp && sp < 0x1_0000) && (0 <= m[sp] && m[sp] < 0x100) && (0 <= m[(sp + 1) % 0x1_0000] && m[(sp + 1) % 0x1_0000] < 0x100))]
        contract_returnfrominterrupt_taken: {
            k: [Instruction::ReturnFromInterrupt],
            sp: sp,
            pc: _,
            f: flags,
            memory: m,
            w: _,
            z: _,
            bus: log,
            ime: _,
        } => {
            k: [],
            sp: ((sp + 2) % 0x1_0000),
            pc: (m[(sp + 1) % 0x1_0000] * 0x100 + m[sp]),
            w: m[(sp + 1) % 0x1_0000],
            z: m[sp],
            bus: [..log, Cycle::Read { addr: sp, value: m[sp] }, Cycle::Read { addr: ((sp + 1) % 0x1_0000), value: m[(sp + 1) % 0x1_0000] }, Cycle::Internal],
            ime: true,
        };

        /// Untaken conditional RET tests flags for one internal cycle and does not access the stack.
        #[k::claim(all_paths)]
        #[k::requires((0 <= flags && flags < 0x100 && flags % 0x10 == 0) && !(kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && flags / 0x80 == 1) || (kequal::eq(cond, Condition::NotZero) && flags / 0x80 == 0) || (kequal::eq(cond, Condition::Carry) && (flags / 0x10) % 2 == 1) || (kequal::eq(cond, Condition::NotCarry) && (flags / 0x10) % 2 == 0)))]
        contract_returnif_not_taken: {
            k: [Instruction::ReturnIf { cond: cond @ (Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            f: flags,
            bus: log,
        } => {
            k: [],
            bus: [..log, Cycle::Internal],
        };

        /// RST: all eight vectors, return PC high then low, wrapping SP and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(0 <= target && target <= 0x38 && target % 8 == 0 && (0 <= pc && pc < 0x1_0000) && (0 <= sp && sp < 0x1_0000) && m.contains_key((sp + 0xFFFF) % 0x1_0000) && m.contains_key((sp + 0xFFFE) % 0x1_0000) && (0 <= m[(sp + 0xFFFF) % 0x1_0000] && m[(sp + 0xFFFF) % 0x1_0000] < 0x100) && (0 <= m[(sp + 0xFFFE) % 0x1_0000] && m[(sp + 0xFFFE) % 0x1_0000] < 0x100))]
        contract_restart: {
            k: [Instruction::Restart { target }],
            pc: pc,
            sp: sp,
            memory: m,
            bus: log,
        } => {
            k: [],
            pc: target,
            sp: ((sp + 0xFFFE) % 0x1_0000),
            memory: k::map![
                ((sp + 0xFFFF) % 0x1_0000) -> pc / 0x100,
                ((sp + 0xFFFE) % 0x1_0000) -> pc % 0x100,
                ..m,
            ],
            bus: [..log, Cycle::Internal, Cycle::Write { addr: ((sp + 0xFFFF) % 0x1_0000), old: m[(sp + 0xFFFF) % 0x1_0000], new: pc / 0x100 }, Cycle::Write { addr: ((sp + 0xFFFE) % 0x1_0000), old: m[(sp + 0xFFFE) % 0x1_0000], new: pc % 0x100 }],
        };
    }
}
