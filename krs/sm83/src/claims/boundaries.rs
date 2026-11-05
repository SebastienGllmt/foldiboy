//! Instruction-boundary and HALT contracts; see README.md for hardware evidence.
//! Behavior: RGBDS gbz80(7), https://rgbds.gbdev.io/docs/gbz80.7 .
//! Encoding/families: Pan Docs CPU Instruction Set; bus order: izik1/gbops dmgops.json.
//! See README.md for pinned sources, coverage and model exclusions.
//! These are authored proof inputs; none calls the implementation's ALU/result helpers.

use crate::Machine;
use crate::alu::{Alu, Unary};
use crate::decode::{Address, Condition, Instruction, Operand, Pair16};
use crate::interrupt::{IE, IF, Step};
use crate::machine::{Event, Register};
use crate::well_formed::{IsByte, IsWord};
use crate::micro::Micro;
use k::builtin::Int;

/// Pan Docs interrupt priority; test requests from bit 0 through bit 4.
#[k::function(total, internal)]
pub fn ExpectedInterruptVector(ie: k::Term<Int>, requests: k::Term<Int>) -> k::Term<Int> {
    if (ie / 1) % 2 == 1 && (requests / 1) % 2 == 1 {
        0x0040
    } else if (ie / 2) % 2 == 1 && (requests / 2) % 2 == 1 {
        0x0048
    } else if (ie / 4) % 2 == 1 && (requests / 4) % 2 == 1 {
        0x0050
    } else if (ie / 8) % 2 == 1 && (requests / 8) % 2 == 1 {
        0x0058
    } else if (ie / 0x10) % 2 == 1 && (requests / 0x10) % 2 == 1 {
        0x0060
    } else {
        0x0000
    }
}

/// Pan Docs interrupt priority; test requests from bit 0 through bit 4.
#[k::function(total, internal)]
pub fn ExpectedInterruptMask(ie: k::Term<Int>, requests: k::Term<Int>) -> k::Term<Int> {
    if (ie / 1) % 2 == 1 && (requests / 1) % 2 == 1 {
        0b0000_0001
    } else if (ie / 2) % 2 == 1 && (requests / 2) % 2 == 1 {
        0b0000_0010
    } else if (ie / 4) % 2 == 1 && (requests / 4) % 2 == 1 {
        0b0000_0100
    } else if (ie / 8) % 2 == 1 && (requests / 8) % 2 == 1 {
        0b0000_1000
    } else if (ie / 0x10) % 2 == 1 && (requests / 0x10) % 2 == 1 {
        0b0001_0000
    } else {
        0
    }
}

#[k::spec_module(name = "SM83-BOUNDARIES-SPEC", imports = [crate])]
pub mod spec {
    use super::*;
    k::claims! {
        for Machine;

        /// Pan Docs HALT: no requested interrupt enters halted mode, independently of IME.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    IsByte(m[IE])
                        && IsByte(m[IF])
                        && !((m[IE] & m[IF] & 0b0001_1111) != 0)
                ),
        )]
        contract_halt_enter: {
            rom_end,
            k: [Instruction::Halt],
            memory: m,
            halted: false,
        } => {
            k: [],
            halted: true,
        };

        /// Pan Docs HALT bug: IME clear with a pending interrupt does not halt; next fetch suppresses increment.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    IsByte(m[IE])
                        && IsByte(m[IF])
                        && ((m[IE] & m[IF] & 0b0001_1111) != 0)
                ),
        )]
        contract_halt_bug_set: {
            rom_end,
            k: [Instruction::Halt],
            ime: false,
            halted: false,
            halt_bug: false,
            memory: m,
        } => {
            k: [],
            halt_bug: true,
        };

        /// SameSuite buffered VBlank/STAT case: delayed IME has taken effect and return PC remains on HALT.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    IsByte(m[IE])
                        && IsByte(m[IF])
                        && ((m[IE] & m[IF] & 0b0001_1111) != 0)
                        && (m[IE] & m[IF] & 0b0000_0011) != 0
                        && IsWord(pc)
                ),
        )]
        contract_halt_pending_ime: {
            rom_end,
            k: [Instruction::Halt],
            pc: pc,
            ime: true,
            ei_pending: false,
            halted: false,
            memory: m,
        } => {
            k: [],
            pc: ((pc + 0xFFFF) % 0x1_0000),
        };

        /// HALT with no request: one idle cycle, no fetch and no architectural state change.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    IsByte(m[IE])
                        && IsByte(m[IF])
                        && !((m[IE] & m[IF] & 0b0001_1111) != 0)
                ),
        )]
        contract_halt_idle: {
            rom_end,
            k: [Step::Boundary],
            halted: true,
            memory: m,
            bus: log,
        } => {
            k: [Step::Boundary],
            bus: [..log, Event::Idle],
        };

        /// Pan Docs HALT: pending request wakes IME=0; fetch the next NOP without acknowledging IF.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    IsByte(m[IE])
                        && IsByte(m[IF])
                        && ((m[IE] & m[IF] & 0b0001_1111) != 0)
                        && IsWord(pc)
                        && m[pc] == 0x00
                ),
        )]
        contract_halt_wake_without_dispatch: {
            rom_end,
            k: [Step::Boundary],
            halted: true,
            ime: false,
            ei_pending: false,
            halt_bug: false,
            memory: m,
            pc: pc,
            bus: log,
        } => {
            k: [Step::Boundary],
            halted: false,
            pc: ((pc + 1) % 0x1_0000),
            bus: [..log, Event::Read { addr: pc, value: 0x00 }],
        };

        /// IME and pending request select dispatch before executing another opcode.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    IsByte(m[IE])
                        && IsByte(m[IF])
                        && ((m[IE] & m[IF] & 0b0001_1111) != 0)
                ),
        )]
        contract_boundary_choose_dispatch: {
            rom_end,
            k: [Step::Boundary],
            halted: false,
            ime: true,
            ei_pending: _,
            memory: m,
        } => {
            k: [Step::Dispatch, Step::Boundary],
            ei_pending: false,
        };

        /// EI delay: old IME prevents dispatch at this boundary, even if a request is pending.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    IsByte(m[IE])
                        && IsByte(m[IF])
                ),
        )]
        contract_boundary_apply_ei_delay: {
            rom_end,
            k: [Step::Boundary],
            halted: false,
            ime: false,
            ei_pending: true,
            memory: m,
        } => {
            k: [Micro::Fetch, Step::Boundary],
            ime: true,
            ei_pending: false,
        };

        /// Five-cycle dispatch: lowest requested bit wins, clear only its IF bit and IME, push PC high then low.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    IsByte(m[IE])
                        && IsByte(m[IF])
                        && ((m[IE] & m[IF] & 0b0001_1111) != 0)
                        && IsWord(pc)
                        && IsWord(sp)
                        && IsByte(m[pc])
                        && m.contains_key((sp + 0xFFFF) % 0x1_0000)
                        && m.contains_key((sp + 0xFFFE) % 0x1_0000)
                        && ((sp + 0xFFFF) % 0x1_0000) != IE
                        && ((sp + 0xFFFF) % 0x1_0000) != IF
                        && ((sp + 0xFFFE) % 0x1_0000) != IE
                        && ((sp + 0xFFFE) % 0x1_0000) != IF
                        && IsByte(m[(sp + 0xFFFF) % 0x1_0000])
                        && IsByte(m[(sp + 0xFFFE) % 0x1_0000])
                ),
        )]
        contract_dispatch_running: {
            rom_end,
            k: [Step::Boundary],
            halted: false,
            ime: true,
            ei_pending: _,
            pc: pc,
            sp: sp,
            memory: m,
            bus: log,
        } => {
            k: [Step::Boundary],
            ime: false,
            ei_pending: false,
            pc: ExpectedInterruptVector(m[IE], m[IF]),
            sp: ((sp + 0xFFFE) % 0x1_0000),
            memory: k::map![
                ((sp + 0xFFFF) % 0x1_0000) -> pc / 0x100,
                ((sp + 0xFFFE) % 0x1_0000) -> pc % 0x100,
                IF -> m[IF] - ExpectedInterruptMask(m[IE], m[IF]),
                ..m,
            ],
            bus: [
                ..log,
                Event::Read { addr: pc, value: m[pc] },
                Event::Idle,
                Event::Idle,
                Event::Write {
                    addr: ((sp + 0xFFFF) % 0x1_0000),
                    value: pc / 0x100,
                },
                Event::Write {
                    addr: ((sp + 0xFFFE) % 0x1_0000),
                    value: pc % 0x100,
                },
            ],
        };

        /// Five-cycle dispatch: lowest requested bit wins, clear only its IF bit and IME, push PC high then low.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    IsByte(m[IE])
                        && IsByte(m[IF])
                        && ((m[IE] & m[IF] & 0b0001_1111) != 0)
                        && IsWord(pc)
                        && IsWord(sp)
                        && IsByte(m[pc])
                        && m.contains_key((sp + 0xFFFF) % 0x1_0000)
                        && m.contains_key((sp + 0xFFFE) % 0x1_0000)
                        && ((sp + 0xFFFF) % 0x1_0000) != IE
                        && ((sp + 0xFFFF) % 0x1_0000) != IF
                        && ((sp + 0xFFFE) % 0x1_0000) != IE
                        && ((sp + 0xFFFE) % 0x1_0000) != IF
                        && IsByte(m[(sp + 0xFFFF) % 0x1_0000])
                        && IsByte(m[(sp + 0xFFFE) % 0x1_0000])
                ),
        )]
        contract_dispatch_wake: {
            rom_end,
            k: [Step::Boundary],
            halted: true,
            ime: true,
            ei_pending: _,
            pc: pc,
            sp: sp,
            memory: m,
            bus: log,
        } => {
            k: [Step::Boundary],
            ime: false,
            ei_pending: false,
            pc: ExpectedInterruptVector(m[IE], m[IF]),
            sp: ((sp + 0xFFFE) % 0x1_0000),
            memory: k::map![
                ((sp + 0xFFFF) % 0x1_0000) -> pc / 0x100,
                ((sp + 0xFFFE) % 0x1_0000) -> pc % 0x100,
                IF -> m[IF] - ExpectedInterruptMask(m[IE], m[IF]),
                ..m,
            ],
            bus: [
                ..log,
                Event::Read { addr: pc, value: m[pc] },
                Event::Idle,
                Event::Idle,
                Event::Write {
                    addr: ((sp + 0xFFFF) % 0x1_0000),
                    value: pc / 0x100,
                },
                Event::Write {
                    addr: ((sp + 0xFFFE) % 0x1_0000),
                    value: pc % 0x100,
                },
            ],
            halted: false,
        };

        /// Mooneye ie_push: PC=0x0200, SP=0x0000 puts the high-byte push on IE.
        /// Writing 0x02 clears the pending timer bit (0b0000_0100), cancelling dispatch.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    m[IE] == 0b0000_0100
                        && m[IF] == 0b0000_0100
                        && IsByte(m[0x0200])
                        && m.contains_key(IE)
                        && m.contains_key(0xFFFE)
                        && IsByte(m[IE])
                        && IsByte(m[0xFFFE])
                ),
        )]
        contract_dispatch_ie_high_cancels: {
            rom_end,
            k: [Step::Dispatch],
            ime: true,
            pc: 0x0200,
            sp: 0x0000,
            memory: m,
            bus: log,
        } => {
            k: [],
            ime: false,
            pc: 0x0000,
            sp: 0xFFFE,
            memory: k::map![IE -> 0x02, 0xFFFE -> 0x00, IF -> 0b0000_0100, ..m],
            bus: [
                ..log,
                Event::Read { addr: 0x0200, value: m[0x0200] },
                Event::Idle,
                Event::Idle,
                Event::Write { addr: IE, value: 0x02 },
                Event::Write { addr: 0xFFFE, value: 0x00 },
            ],
        };

        /// Mooneye ie_push: PC=0x0235, SP=0x0001 puts the low-byte push on IE.
        /// Writing 0x35 clears the serial bit (0b0000_1000), but IE was already sampled.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    m[IE] == 0b0000_1000
                        && m[IF] == 0b0000_1000
                        && IsByte(m[0x0235])
                        && m.contains_key(0x0000)
                        && m.contains_key(IE)
                        && IsByte(m[0x0000])
                        && IsByte(m[IE])
                ),
        )]
        contract_dispatch_ie_low_too_late: {
            rom_end,
            k: [Step::Dispatch],
            ime: true,
            pc: 0x0235,
            sp: 0x0001,
            memory: m,
            bus: log,
        } => {
            k: [],
            ime: false,
            pc: 0x0058,
            sp: 0xFFFF,
            memory: k::map![0x0000 -> 0x02, IE -> 0x35, IF -> 0x00, ..m],
            bus: [
                ..log,
                Event::Read { addr: 0x0235, value: m[0x0235] },
                Event::Idle,
                Event::Idle,
                Event::Write { addr: 0x0000, value: 0x02 },
                Event::Write { addr: IE, value: 0x35 },
            ],
        };

        /// Pan Docs and mooneye EI timing: one following instruction executes; DI cancels the delayed enable.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    m[IE] == 0b0000_0001
                        && m[IF] == 0b0000_0001
                        && m[0x0100] == 0xFB
                        && m[0x0101] == 0x00
                ),
        )]
        contract_ei_then_nop: {
            rom_end,
            k: [Step::Boundary],
            pc: 0x0100,
            ime: false,
            ei_pending: false,
            halted: false,
            halt_bug: false,
            memory: m,
            bus: log,
        } => {
            k: [Step::Boundary],
            pc: 0x0102,
            ime: true,
            bus: [
                ..log,
                Event::Read { addr: 0x0100, value: 0xFB },
                Event::Read { addr: 0x0101, value: 0x00 },
            ],
        };

        /// Pan Docs and mooneye EI timing: one following instruction executes; DI cancels the delayed enable.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    m[IE] == 0b0000_0001
                        && m[IF] == 0b0000_0001
                        && m[0x0100] == 0xFB
                        && m[0x0101] == 0xF3
                ),
        )]
        contract_ei_then_di: {
            rom_end,
            k: [Step::Boundary],
            pc: 0x0100,
            ime: false,
            ei_pending: false,
            halted: false,
            halt_bug: false,
            memory: m,
            bus: log,
        } => {
            k: [Step::Boundary],
            pc: 0x0102,
            ime: false,
            bus: [
                ..log,
                Event::Read { addr: 0x0100, value: 0xFB },
                Event::Read { addr: 0x0101, value: 0xF3 },
            ],
        };

        /// SameSuite ei_delay_halt: prebuffered VBlank/STAT, EI followed by HALT, return PC remains on HALT; no timer arrival modeled.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    m[IE] == 0b0000_0111
                        && m[IF] == 0b0000_0011
                        && m[0x0100] == 0xFB
                        && m[0x0101] == 0x76
                ),
        )]
        contract_ei_then_halt_buffered: {
            rom_end,
            k: [Step::Boundary],
            pc: 0x0100,
            ime: false,
            ei_pending: false,
            halted: false,
            halt_bug: false,
            memory: m,
            bus: log,
        } => {
            k: [Step::Boundary],
            pc: 0x0101,
            ime: true,
            bus: [
                ..log,
                Event::Read { addr: 0x0100, value: 0xFB },
                Event::Read { addr: 0x0101, value: 0x76 },
            ],
        };
    }
}
