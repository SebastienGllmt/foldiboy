//! Fetch-to-finish composition contracts over symbolic memory, with wrapping PC.
//! Behavior: RGBDS gbz80(7), https://rgbds.gbdev.io/docs/gbz80.7 .
//! Encoding/families: Pan Docs CPU Instruction Set; bus order: izik1/gbops dmgops.json.
//! See README.md for pinned sources, coverage and model exclusions.
//! These are authored proof inputs; none calls the implementation's ALU/result helpers.

use crate::Machine;
use crate::alu::{Alu, Flags, Unary};
use crate::decode::{Address, Condition, Instruction, Operand, Pair16};
use crate::interrupt::{IE, IF, Step};
use crate::machine::{Event, Register};
use crate::micro::Micro;
use k::builtin::{int, kequal};

#[k::spec_module(name = "SM83-FETCH-SPEC", imports = [crate])]
pub mod spec {
    use super::*;
    k::claims! {
        for Machine;

        /// NOP: full opcode fetch, wrapping PC and preservation of every other cell.
        #[k::claim(all_paths)]
        #[k::requires((0 <= pc && pc < 0x1_0000) && m[pc] == 0x00)]
        contract_fetch_nop: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            memory: m,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 1) % 0x1_0000),
            bus: [..log, Event::Read { addr: pc, value: 0x00 }],
        };

        /// LD r8,d8: independent opcode encoding, operand read, final PC and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x06
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_b_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            regs: r,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            regs: k::map![Register::B -> m[(pc + 1) % 0x1_0000], ..r],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x06 },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
            ],
        };

        /// LD r8,d8: independent opcode encoding, operand read, final PC and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x0E
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_c_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            regs: r,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            regs: k::map![Register::C -> m[(pc + 1) % 0x1_0000], ..r],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x0E },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
            ],
        };

        /// LD r8,d8: independent opcode encoding, operand read, final PC and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x16
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_d_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            regs: r,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            regs: k::map![Register::D -> m[(pc + 1) % 0x1_0000], ..r],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x16 },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
            ],
        };

        /// LD r8,d8: independent opcode encoding, operand read, final PC and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x1E
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_e_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            regs: r,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            regs: k::map![Register::E -> m[(pc + 1) % 0x1_0000], ..r],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x1E },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
            ],
        };

        /// LD r8,d8: independent opcode encoding, operand read, final PC and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x26
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_h_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            regs: r,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            regs: k::map![Register::H -> m[(pc + 1) % 0x1_0000], ..r],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x26 },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
            ],
        };

        /// LD r8,d8: independent opcode encoding, operand read, final PC and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x2E
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_l_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            regs: r,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            regs: k::map![Register::L -> m[(pc + 1) % 0x1_0000], ..r],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x2E },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
            ],
        };

        /// LD r8,d8: independent opcode encoding, operand read, final PC and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x3E
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_a_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            regs: r,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            regs: k::map![Register::A -> m[(pc + 1) % 0x1_0000], ..r],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x3E },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
            ],
        };

        /// LD r16,d16: three reads, little-endian assembly and PC wrapping, flags unchanged.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x01
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100)
                && (0 <= m[(pc + 2) % 0x1_0000] && m[(pc + 2) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_bc_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            memory: m,
            w: _,
            z: _,
            bus: log,
            regs: r,
        } => {
            k: [],
            pc: ((pc + 3) % 0x1_0000),
            w: m[(pc + 2) % 0x1_0000],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x01 },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
                Event::Read { addr: ((pc + 2) % 0x1_0000), value: m[(pc + 2) % 0x1_0000] },
            ],
            regs: k::map![
                Register::B -> m[(pc + 2) % 0x1_0000],
                Register::C -> m[(pc + 1) % 0x1_0000],
                ..r,
            ],
        };

        /// LD r16,d16: three reads, little-endian assembly and PC wrapping, flags unchanged.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x11
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100)
                && (0 <= m[(pc + 2) % 0x1_0000] && m[(pc + 2) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_de_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            memory: m,
            w: _,
            z: _,
            bus: log,
            regs: r,
        } => {
            k: [],
            pc: ((pc + 3) % 0x1_0000),
            w: m[(pc + 2) % 0x1_0000],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x11 },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
                Event::Read { addr: ((pc + 2) % 0x1_0000), value: m[(pc + 2) % 0x1_0000] },
            ],
            regs: k::map![
                Register::D -> m[(pc + 2) % 0x1_0000],
                Register::E -> m[(pc + 1) % 0x1_0000],
                ..r,
            ],
        };

        /// LD r16,d16: three reads, little-endian assembly and PC wrapping, flags unchanged.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x21
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100)
                && (0 <= m[(pc + 2) % 0x1_0000] && m[(pc + 2) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_hl_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            memory: m,
            w: _,
            z: _,
            bus: log,
            regs: r,
        } => {
            k: [],
            pc: ((pc + 3) % 0x1_0000),
            w: m[(pc + 2) % 0x1_0000],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x21 },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
                Event::Read { addr: ((pc + 2) % 0x1_0000), value: m[(pc + 2) % 0x1_0000] },
            ],
            regs: k::map![
                Register::H -> m[(pc + 2) % 0x1_0000],
                Register::L -> m[(pc + 1) % 0x1_0000],
                ..r,
            ],
        };

        /// LD r16,d16: three reads, little-endian assembly and PC wrapping, flags unchanged.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x31
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100)
                && (0 <= m[(pc + 2) % 0x1_0000] && m[(pc + 2) % 0x1_0000] < 0x100),
        )]
        contract_fetch_ld_sp_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            memory: m,
            w: _,
            z: _,
            bus: log,
            sp: _,
        } => {
            k: [],
            pc: ((pc + 3) % 0x1_0000),
            w: m[(pc + 2) % 0x1_0000],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x31 },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
                Event::Read { addr: ((pc + 2) % 0x1_0000), value: m[(pc + 2) % 0x1_0000] },
            ],
            sp: (m[(pc + 2) % 0x1_0000] * 0x100 + m[(pc + 1) % 0x1_0000]),
        };

        /// Pan Docs HALT bug: consume the bug flag and suppress precisely this opcode-fetch increment.
        #[k::claim(all_paths)]
        #[k::requires((0 <= pc && pc < 0x1_0000) && m[pc] == 0x00)]
        contract_halt_bug_fetch_nop: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: true,
            memory: m,
            bus: log,
        } => {
            k: [],
            halt_bug: false,
            bus: [..log, Event::Read { addr: pc, value: 0x00 }],
        };

        /// HALT bug followed by LD B,d8: opcode is reread as the immediate byte.
        #[k::claim(all_paths)]
        #[k::requires((0 <= pc && pc < 0x1_0000) && m[pc] == 0x06)]
        contract_halt_bug_repeats_immediate: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: true,
            regs: r,
            memory: m,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 1) % 0x1_0000),
            halt_bug: false,
            regs: k::map![Register::B -> 6, ..r],
            z: 6,
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x06 },
                Event::Read { addr: pc, value: 0x06 },
            ],
        };

        /// CB RLC A: prefix consumes its own fetch; unlike RLCA, zero follows the result.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0xCB
                && m[(pc + 1) % 0x1_0000] == 0x07
                && (0 <= r[Register::A] && r[Register::A] < 0x100),
        )]
        contract_fetch_cb_rlc_a: {
            k: [Micro::Fetch],
            pc: pc,
            halt_bug: false,
            regs: r,
            memory: m,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            regs: k::map![
                Register::A -> ((r[Register::A] * 2) % 0x100 + r[Register::A] / 0x80),
                ..r,
            ],
            f: Flags {
                z: (r[Register::A] * 2) % 0x100 + r[Register::A] / 0x80 == 0,
                n: false,
                h: false,
                c: r[Register::A] / 0x80 == 1,
            },
            z: ((r[Register::A] * 2) % 0x100 + r[Register::A] / 0x80),
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0xCB },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: 0x07 },
            ],
        };

        /// LD HL,d16 from the boundary, with no interrupt requested: arbitration, fetch, decode and execution compose.
        #[k::claim(all_paths)]
        #[k::requires(
            (0 <= pc && pc < 0x1_0000)
                && m[pc] == 0x21
                && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100)
                && (0 <= m[(pc + 2) % 0x1_0000] && m[(pc + 2) % 0x1_0000] < 0x100)
                && m[IE] == 0
                && m[IF] == 0,
        )]
        contract_boundary_ld_hl_immediate: {
            k: [Step::Boundary],
            pc: pc,
            halt_bug: false,
            halted: false,
            ime: false,
            ei_pending: false,
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
        } => {
            k: [Step::Boundary],
            pc: ((pc + 3) % 0x1_0000),
            regs: k::map![
                Register::H -> m[(pc + 2) % 0x1_0000],
                Register::L -> m[(pc + 1) % 0x1_0000],
                ..r,
            ],
            w: m[(pc + 2) % 0x1_0000],
            z: m[(pc + 1) % 0x1_0000],
            bus: [
                ..log,
                Event::Read { addr: pc, value: 0x21 },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
                Event::Read { addr: ((pc + 2) % 0x1_0000), value: m[(pc + 2) % 0x1_0000] },
            ],
        };
    }
}
