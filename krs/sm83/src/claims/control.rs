//! Branch, call, return and restart contracts, after opcode fetch.
//! Behavior: RGBDS gbz80(7), https://rgbds.gbdev.io/docs/gbz80.7 .
//! Encoding/families: Pan Docs CPU Instruction Set; bus order: izik1/gbops dmgops.json.
//! See README.md for pinned sources, coverage and model exclusions.
//! These are authored proof inputs; none calls the implementation's ALU/result helpers.

use k::builtin::{int, kequal};
use crate::decode::{Address, Condition, Instruction, Operand, Pair16};
use crate::alu::{Alu, Flags, Unary};
use crate::machine::{Event, Register};
use crate::well_formed::{IsByte, IsWord};
use crate::Machine;

#[k::spec_module(name = "SM83-CONTROL-SPEC", imports = [crate])]
pub mod spec {
    use super::*;
    k::claims! {
        for Machine;

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires(rom_end == 0 && (IsWord(pc) && IsByte(m[pc]) && (kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && zero) || (kequal::eq(cond, Condition::NotZero) && !zero) || (kequal::eq(cond, Condition::Carry) && carry) || (kequal::eq(cond, Condition::NotCarry) && !carry))))]
        contract_jumprelative_taken: {
            rom_end,
            k: [Instruction::JumpRelative { cond: cond @ (Condition::Always | Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: Flags { z: zero, c: carry, .. },
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            z: m[pc],
            pc: int::rem_euclid(pc + 1 + kequal::ite(m[pc] < 0x80, m[pc], m[pc] - 0x100), 0x1_0000),
            bus: [..log, Event::Read { addr: pc, value: m[pc] }, Event::Idle],
        };

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires(rom_end == 0 && (IsWord(pc) && IsByte(m[pc]) && !(kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && zero) || (kequal::eq(cond, Condition::NotZero) && !zero) || (kequal::eq(cond, Condition::Carry) && carry) || (kequal::eq(cond, Condition::NotCarry) && !carry))))]
        contract_jumprelative_not_taken: {
            rom_end,
            k: [Instruction::JumpRelative { cond: cond @ (Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: Flags { z: zero, c: carry, .. },
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            z: m[pc],
            pc: ((pc + 1) % 0x1_0000),
            bus: [..log, Event::Read { addr: pc, value: m[pc] }],
        };

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires(rom_end == 0 && (IsWord(pc) && IsByte(m[pc]) && (kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && zero) || (kequal::eq(cond, Condition::NotZero) && !zero) || (kequal::eq(cond, Condition::Carry) && carry) || (kequal::eq(cond, Condition::NotCarry) && !carry)) && IsByte(m[(pc + 1) % 0x1_0000])))]
        contract_jump_taken: {
            rom_end,
            k: [Instruction::Jump { cond: cond @ (Condition::Always | Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: Flags { z: zero, c: carry, .. },
            memory: m,
            z: _,
            bus: log,
            w: _,
        } => {
            k: [],
            z: m[pc],
            w: m[(pc + 1) % 0x1_0000],
            pc: (m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]),
            bus: [..log, Event::Read { addr: pc, value: m[pc] }, Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] }, Event::Idle],
        };

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires(rom_end == 0 && (IsWord(pc) && IsByte(m[pc]) && !(kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && zero) || (kequal::eq(cond, Condition::NotZero) && !zero) || (kequal::eq(cond, Condition::Carry) && carry) || (kequal::eq(cond, Condition::NotCarry) && !carry)) && IsByte(m[(pc + 1) % 0x1_0000])))]
        contract_jump_not_taken: {
            rom_end,
            k: [Instruction::Jump { cond: cond @ (Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: Flags { z: zero, c: carry, .. },
            memory: m,
            z: _,
            bus: log,
            w: _,
        } => {
            k: [],
            z: m[pc],
            w: m[(pc + 1) % 0x1_0000],
            pc: ((pc + 2) % 0x1_0000),
            bus: [..log, Event::Read { addr: pc, value: m[pc] }, Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] }],
        };

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires(rom_end == 0 && (IsWord(pc) && IsByte(m[pc]) && (kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && zero) || (kequal::eq(cond, Condition::NotZero) && !zero) || (kequal::eq(cond, Condition::Carry) && carry) || (kequal::eq(cond, Condition::NotCarry) && !carry)) && IsByte(m[(pc + 1) % 0x1_0000]) && IsWord(sp) && m.contains_key((sp + 0xFFFF) % 0x1_0000) && m.contains_key((sp + 0xFFFE) % 0x1_0000) && IsByte(m[(sp + 0xFFFF) % 0x1_0000]) && IsByte(m[(sp + 0xFFFE) % 0x1_0000])))]
        contract_call_taken: {
            rom_end,
            k: [Instruction::Call { cond: cond @ (Condition::Always | Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: Flags { z: zero, c: carry, .. },
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
            bus: [..log, Event::Read { addr: pc, value: m[pc] }, Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] }, Event::Idle, Event::Write { addr: ((sp + 0xFFFF) % 0x1_0000), value: ((pc + 2) % 0x1_0000) / 0x100 }, Event::Write { addr: ((sp + 0xFFFE) % 0x1_0000), value: ((pc + 2) % 0x1_0000) % 0x100 }],
        };

        /// RGBDS conditional control flow: taken condition, PC result, stack effects and exact bus order.
        #[k::claim(all_paths)]
        #[k::requires(rom_end == 0 && (IsWord(pc) && IsByte(m[pc]) && !(kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && zero) || (kequal::eq(cond, Condition::NotZero) && !zero) || (kequal::eq(cond, Condition::Carry) && carry) || (kequal::eq(cond, Condition::NotCarry) && !carry)) && IsByte(m[(pc + 1) % 0x1_0000])))]
        contract_call_not_taken: {
            rom_end,
            k: [Instruction::Call { cond: cond @ (Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            pc: pc,
            f: Flags { z: zero, c: carry, .. },
            memory: m,
            z: _,
            bus: log,
            w: _,
        } => {
            k: [],
            z: m[pc],
            w: m[(pc + 1) % 0x1_0000],
            pc: ((pc + 2) % 0x1_0000),
            bus: [..log, Event::Read { addr: pc, value: m[pc] }, Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] }],
        };

        /// RET/RETI: pop low then high, final internal cycle; RETI enables IME immediately.
        #[k::claim(all_paths)]
        #[k::requires(rom_end == 0 && (IsWord(sp) && IsByte(m[sp]) && IsByte(m[(sp + 1) % 0x1_0000])))]
        contract_return_taken: {
            rom_end,
            k: [Instruction::Return],
            sp: sp,
            pc: _,
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
            bus: [..log, Event::Read { addr: sp, value: m[sp] }, Event::Read { addr: ((sp + 1) % 0x1_0000), value: m[(sp + 1) % 0x1_0000] }, Event::Idle],
        };

        /// RET/RETI: pop low then high, final internal cycle; RETI enables IME immediately.
        #[k::claim(all_paths)]
        #[k::requires(rom_end == 0 && (IsWord(sp) && IsByte(m[sp]) && IsByte(m[(sp + 1) % 0x1_0000]) && (kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && zero) || (kequal::eq(cond, Condition::NotZero) && !zero) || (kequal::eq(cond, Condition::Carry) && carry) || (kequal::eq(cond, Condition::NotCarry) && !carry))))]
        contract_returnif_taken: {
            rom_end,
            k: [Instruction::ReturnIf { cond: cond @ (Condition::Always | Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            sp: sp,
            pc: _,
            f: Flags { z: zero, c: carry, .. },
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
            bus: [..log, Event::Idle, Event::Read { addr: sp, value: m[sp] }, Event::Read { addr: ((sp + 1) % 0x1_0000), value: m[(sp + 1) % 0x1_0000] }, Event::Idle],
        };

        /// RET/RETI: pop low then high, final internal cycle; RETI enables IME immediately.
        #[k::claim(all_paths)]
        #[k::requires(rom_end == 0 && (IsWord(sp) && IsByte(m[sp]) && IsByte(m[(sp + 1) % 0x1_0000])))]
        contract_returnfrominterrupt_taken: {
            rom_end,
            k: [Instruction::ReturnFromInterrupt],
            sp: sp,
            pc: _,
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
            bus: [..log, Event::Read { addr: sp, value: m[sp] }, Event::Read { addr: ((sp + 1) % 0x1_0000), value: m[(sp + 1) % 0x1_0000] }, Event::Idle],
            ime: true,
        };

        /// Untaken conditional RET tests flags for one internal cycle and does not access the stack.
        #[k::claim(all_paths)]
        #[k::requires(!(kequal::eq(cond, Condition::Always) || (kequal::eq(cond, Condition::Zero) && zero) || (kequal::eq(cond, Condition::NotZero) && !zero) || (kequal::eq(cond, Condition::Carry) && carry) || (kequal::eq(cond, Condition::NotCarry) && !carry)))]
        contract_returnif_not_taken: {
            k: [Instruction::ReturnIf { cond: cond @ (Condition::NotZero | Condition::Zero | Condition::NotCarry | Condition::Carry) }],
            f: Flags { z: zero, c: carry, .. },
            bus: log,
        } => {
            k: [],
            bus: [..log, Event::Idle],
        };

        /// RST: all eight vectors, return PC high then low, wrapping SP and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(rom_end == 0 && (0 <= target && target <= 0x38 && target % 8 == 0 && IsWord(pc) && IsWord(sp) && m.contains_key((sp + 0xFFFF) % 0x1_0000) && m.contains_key((sp + 0xFFFE) % 0x1_0000) && IsByte(m[(sp + 0xFFFF) % 0x1_0000]) && IsByte(m[(sp + 0xFFFE) % 0x1_0000])))]
        contract_restart: {
            rom_end,
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
            bus: [..log, Event::Idle, Event::Write { addr: ((sp + 0xFFFF) % 0x1_0000), value: pc / 0x100 }, Event::Write { addr: ((sp + 0xFFFE) % 0x1_0000), value: pc % 0x100 }],
        };
    }
}
