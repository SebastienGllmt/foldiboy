//! Addressing, stack and signed-SP instruction contracts, after opcode fetch.
//! Behavior: RGBDS gbz80(7), https://rgbds.gbdev.io/docs/gbz80.7 .
//! Encoding/families: Pan Docs CPU Instruction Set; bus order: izik1/gbops dmgops.json.
//! See README.md for pinned sources, coverage and model exclusions.
//! These are authored proof inputs; none calls the implementation's ALU/result helpers.

use crate::Machine;
use crate::alu::{Alu, Flags, Unary};
use crate::decode::{Address, Condition, Instruction, Operand, Pair16};
use crate::machine::{Event, Register};
use k::builtin::{Bool, Int, int, kequal};

/// RGBDS PUSH AF: the F byte holds Z, N, H, C in bits 7 to 4 and zero in bits 3 to 0.
#[k::function(total, internal)]
pub fn ExpectedFlagByte(
    zero: k::Term<Bool>,
    subtract: k::Term<Bool>,
    half_carry: k::Term<Bool>,
    carry: k::Term<Bool>,
) -> k::Term<Int> {
    0x80 * kequal::ite(zero, 1, 0)
        + 0x40 * kequal::ite(subtract, 1, 0)
        + 0x20 * kequal::ite(half_carry, 1, 0)
        + 0x10 * kequal::ite(carry, 1, 0)
}

#[k::spec_module(name = "SM83-MEMORY-SPEC", imports = [crate])]
pub mod spec {
    use super::*;
    k::claims! {
        for Machine;

        /// LD through A: specified address calculation, HL step where present, bus order and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= r[high] && r[high] < 0x100)
                        && (0 <= r[low] && r[low] < 0x100)
                        && (0 <= m[r[high] * 0x100 + r[low]]
                            && m[r[high] * 0x100 + r[low]] < 0x100)
                ),
        )]
        contract_load_a_pair: {
            rom_end,
            k: [Instruction::LoadA { addr: Address::ViaPair { high, low } }],
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
        } => {
            k: [],
            w: r[high],
            z: m[r[high] * 0x100 + r[low]],
            regs: k::map![Register::A -> m[r[high] * 0x100 + r[low]], ..r],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[high] * 0x100 + r[low]),
                    value: m[r[high] * 0x100 + r[low]],
                },
            ],
        };

        /// LD through A: specified address calculation, HL step where present, bus order and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= r[high] && r[high] < 0x100)
                        && (0 <= r[low] && r[low] < 0x100)
                        && (0 <= r[Register::A] && r[Register::A] < 0x100)
                        && m.contains_key(r[high] * 0x100 + r[low])
                        && (0 <= m[r[high] * 0x100 + r[low]]
                            && m[r[high] * 0x100 + r[low]] < 0x100)
                ),
        )]
        contract_store_a_pair: {
            rom_end,
            k: [Instruction::StoreA { addr: Address::ViaPair { high, low } }],
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
        } => {
            k: [],
            w: r[high],
            z: r[low],
            memory: k::map![(r[high] * 0x100 + r[low]) -> r[Register::A], ..m],
            bus: [
                ..log,
                Event::Write {
                    addr: (r[high] * 0x100 + r[low]),
                    value: r[Register::A],
                },
            ],
        };

        /// LD through A: specified address calculation, HL step where present, bus order and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= r[Register::H] && r[Register::H] < 0x100)
                        && (0 <= r[Register::L] && r[Register::L] < 0x100)
                        && (delta == 1 || delta == -1)
                        && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                            && m[r[Register::H] * 0x100 + r[Register::L]]
                                < 0x100)
                ),
        )]
        contract_load_a_hl_step: {
            rom_end,
            k: [Instruction::LoadA { addr: Address::ViaHlStep { delta } }],
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
        } => {
            k: [],
            w: r[Register::H],
            z: m[r[Register::H] * 0x100 + r[Register::L]],
            regs: k::map![
                Register::H -> (
                    ((r[Register::H] * 0x100 + r[Register::L]) + 0x1_0000 + delta) % 0x1_0000
                ) / 0x100,
                Register::L -> (
                    ((r[Register::H] * 0x100 + r[Register::L]) + 0x1_0000 + delta) % 0x1_0000
                ) % 0x100,
                Register::A -> m[r[Register::H] * 0x100 + r[Register::L]],
                ..r,
            ],
            bus: [
                ..log,
                Event::Read {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: m[r[Register::H] * 0x100 + r[Register::L]],
                },
            ],
        };

        /// LD through A: specified address calculation, HL step where present, bus order and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= r[Register::H] && r[Register::H] < 0x100)
                        && (0 <= r[Register::L] && r[Register::L] < 0x100)
                        && (delta == 1 || delta == -1)
                        && (0 <= r[Register::A] && r[Register::A] < 0x100)
                        && m.contains_key(r[Register::H] * 0x100 + r[Register::L])
                        && (0 <= m[r[Register::H] * 0x100 + r[Register::L]]
                            && m[r[Register::H] * 0x100 + r[Register::L]]
                                < 0x100)
                ),
        )]
        contract_store_a_hl_step: {
            rom_end,
            k: [Instruction::StoreA { addr: Address::ViaHlStep { delta } }],
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
        } => {
            k: [],
            w: r[Register::H],
            z: r[Register::L],
            regs: k::map![
                Register::H -> (
                    ((r[Register::H] * 0x100 + r[Register::L]) + 0x1_0000 + delta) % 0x1_0000
                ) / 0x100,
                Register::L -> (
                    ((r[Register::H] * 0x100 + r[Register::L]) + 0x1_0000 + delta) % 0x1_0000
                ) % 0x100,
                ..r,
            ],
            memory: k::map![(r[Register::H] * 0x100 + r[Register::L]) -> r[Register::A], ..m],
            bus: [
                ..log,
                Event::Write {
                    addr: (r[Register::H] * 0x100 + r[Register::L]),
                    value: r[Register::A],
                },
            ],
        };

        /// LD through A: specified address calculation, HL step where present, bus order and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= r[Register::C] && r[Register::C] < 0x100)
                        && (0 <= m[0xFF00 + r[Register::C]]
                            && m[0xFF00 + r[Register::C]] < 0x100)
                ),
        )]
        contract_load_a_high_c: {
            rom_end,
            k: [Instruction::LoadA { addr: Address::HighC }],
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
        } => {
            k: [],
            w: 0xFF,
            z: m[0xFF00 + r[Register::C]],
            regs: k::map![Register::A -> m[0xFF00 + r[Register::C]], ..r],
            bus: [
                ..log,
                Event::Read { addr: (0xFF00 + r[Register::C]), value: m[0xFF00 + r[Register::C]] },
            ],
        };

        /// LD through A: specified address calculation, HL step where present, bus order and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= r[Register::C] && r[Register::C] < 0x100)
                        && (0 <= r[Register::A] && r[Register::A] < 0x100)
                        && m.contains_key(0xFF00 + r[Register::C])
                        && (0 <= m[0xFF00 + r[Register::C]]
                            && m[0xFF00 + r[Register::C]] < 0x100)
                ),
        )]
        contract_store_a_high_c: {
            rom_end,
            k: [Instruction::StoreA { addr: Address::HighC }],
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
        } => {
            k: [],
            w: 0xFF,
            z: r[Register::C],
            memory: k::map![(0xFF00 + r[Register::C]) -> r[Register::A], ..m],
            bus: [
                ..log,
                Event::Write {
                    addr: (0xFF00 + r[Register::C]),
                    value: r[Register::A],
                },
            ],
        };

        /// LD through A: specified address calculation, HL step where present, bus order and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= pc && pc < 0x1_0000)
                        && (0 <= m[pc] && m[pc] < 0x100)
                        && (0 <= m[0xFF00 + m[pc]]
                            && m[0xFF00 + m[pc]] < 0x100)
                ),
        )]
        contract_load_a_high_immediate: {
            rom_end,
            k: [Instruction::LoadA { addr: Address::HighImmediate }],
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
            pc: pc,
        } => {
            k: [],
            w: 0xFF,
            z: m[0xFF00 + m[pc]],
            regs: k::map![Register::A -> m[0xFF00 + m[pc]], ..r],
            bus: [
                ..log,
                Event::Read { addr: pc, value: m[pc] },
                Event::Read { addr: (0xFF00 + m[pc]), value: m[0xFF00 + m[pc]] },
            ],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// LD through A: specified address calculation, HL step where present, bus order and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= pc && pc < 0x1_0000)
                        && (0 <= m[pc] && m[pc] < 0x100)
                        && (0 <= r[Register::A] && r[Register::A] < 0x100)
                        && m.contains_key(0xFF00 + m[pc])
                        && (0 <= m[0xFF00 + m[pc]]
                            && m[0xFF00 + m[pc]] < 0x100)
                ),
        )]
        contract_store_a_high_immediate: {
            rom_end,
            k: [Instruction::StoreA { addr: Address::HighImmediate }],
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
            pc: pc,
        } => {
            k: [],
            w: 0xFF,
            z: m[pc],
            memory: k::map![(0xFF00 + m[pc]) -> r[Register::A], ..m],
            bus: [
                ..log,
                Event::Read { addr: pc, value: m[pc] },
                Event::Write {
                    addr: (0xFF00 + m[pc]),
                    value: r[Register::A],
                },
            ],
            pc: ((pc + 1) % 0x1_0000),
        };

        /// LD through A: specified address calculation, HL step where present, bus order and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= pc && pc < 0x1_0000)
                        && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100)
                        && (0 <= m[pc] && m[pc] < 0x100)
                        && (0 <= m[m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]]
                            && m[m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]]
                                < 0x100)
                ),
        )]
        contract_load_a_absolute: {
            rom_end,
            k: [Instruction::LoadA { addr: Address::Absolute }],
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
            pc: pc,
        } => {
            k: [],
            w: m[(pc + 1) % 0x1_0000],
            z: m[m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]],
            regs: k::map![Register::A -> m[m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]], ..r],
            bus: [
                ..log,
                Event::Read { addr: pc, value: m[pc] },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
                Event::Read {
                    addr: (m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]),
                    value: m[m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]],
                },
            ],
            pc: ((pc + 2) % 0x1_0000),
        };

        /// LD through A: specified address calculation, HL step where present, bus order and preserved flags.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= pc && pc < 0x1_0000)
                        && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100)
                        && (0 <= m[pc] && m[pc] < 0x100)
                        && (0 <= r[Register::A] && r[Register::A] < 0x100)
                        && m.contains_key(m[(pc + 1) % 0x1_0000] * 0x100 + m[pc])
                        && (0 <= m[m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]]
                            && m[m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]]
                                < 0x100)
                ),
        )]
        contract_store_a_absolute: {
            rom_end,
            k: [Instruction::StoreA { addr: Address::Absolute }],
            regs: r,
            memory: m,
            w: _,
            z: _,
            bus: log,
            pc: pc,
        } => {
            k: [],
            w: m[(pc + 1) % 0x1_0000],
            z: m[pc],
            memory: k::map![(m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]) -> r[Register::A], ..m],
            bus: [
                ..log,
                Event::Read { addr: pc, value: m[pc] },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
                Event::Write {
                    addr: (m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]),
                    value: r[Register::A],
                },
            ],
            pc: ((pc + 2) % 0x1_0000),
        };

        /// LD [d16],SP: read address little-endian, then write SP low byte before high byte, wrapping addresses.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= pc && pc < 0x1_0000)
                        && (0 <= sp && sp < 0x1_0000)
                        && (0 <= m[pc] && m[pc] < 0x100)
                        && (0 <= m[(pc + 1) % 0x1_0000] && m[(pc + 1) % 0x1_0000] < 0x100)
                        && m.contains_key(m[(pc + 1) % 0x1_0000] * 0x100 + m[pc])
                        && m.contains_key(((m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]) + 1) % 0x1_0000)
                        && (0 <= m[m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]]
                            && m[m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]] < 0x100)
                        && (0
                            <= m[((m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]) + 1) % 0x1_0000]
                            && m[((m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]) + 1) % 0x1_0000] < 0x100)
                ),
        )]
        contract_store_sp: {
            rom_end,
            k: [Instruction::StoreSp],
            pc: pc,
            sp: sp,
            memory: m,
            w: _,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 2) % 0x1_0000),
            memory: k::map![
                (m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]) -> sp % 0x100,
                (((m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]) + 1) % 0x1_0000) -> sp / 0x100,
                ..m,
            ],
            w: m[(pc + 1) % 0x1_0000],
            z: m[pc],
            bus: [
                ..log,
                Event::Read { addr: pc, value: m[pc] },
                Event::Read { addr: ((pc + 1) % 0x1_0000), value: m[(pc + 1) % 0x1_0000] },
                Event::Write {
                    addr: (m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]),
                    value: sp % 0x100,
                },
                Event::Write {
                    addr: (((m[(pc + 1) % 0x1_0000] * 0x100 + m[pc]) + 1) % 0x1_0000),
                    value: sp / 0x100,
                },
            ],
        };

        /// Stack transfer: wrapping SP, hardware byte order, preserved state; POP AF masks the low flag nibble.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= sp && sp < 0x1_0000)
                        && (0 <= r[high] && r[high] < 0x100)
                        && (0 <= r[low] && r[low] < 0x100)
                        && m.contains_key((sp + 0xFFFF) % 0x1_0000)
                        && m.contains_key((sp + 0xFFFE) % 0x1_0000)
                        && (0 <= m[(sp + 0xFFFF) % 0x1_0000]
                            && m[(sp + 0xFFFF) % 0x1_0000] < 0x100)
                        && (0 <= m[(sp + 0xFFFE) % 0x1_0000]
                            && m[(sp + 0xFFFE) % 0x1_0000] < 0x100)
                ),
        )]
        contract_push_registers: {
            rom_end,
            k: [Instruction::Push { pair: Pair16::Registers { high, low } }],
            regs: r,
            sp: sp,
            memory: m,
            bus: log,
        } => {
            k: [],
            sp: ((sp + 0xFFFE) % 0x1_0000),
            memory: k::map![
                ((sp + 0xFFFF) % 0x1_0000) -> r[high],
                ((sp + 0xFFFE) % 0x1_0000) -> r[low],
                ..m,
            ],
            bus: [
                ..log,
                Event::Idle,
                Event::Write {
                    addr: ((sp + 0xFFFF) % 0x1_0000),
                    value: r[high],
                },
                Event::Write {
                    addr: ((sp + 0xFFFE) % 0x1_0000),
                    value: r[low],
                },
            ],
        };

        /// Stack transfer: wrapping SP, hardware byte order, preserved state; POP AF masks the low flag nibble.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= sp && sp < 0x1_0000)
                        && (0 <= r[Register::A] && r[Register::A] < 0x100)
                        && m.contains_key((sp + 0xFFFF) % 0x1_0000)
                        && m.contains_key((sp + 0xFFFE) % 0x1_0000)
                        && (0 <= m[(sp + 0xFFFF) % 0x1_0000]
                            && m[(sp + 0xFFFF) % 0x1_0000] < 0x100)
                        && (0 <= m[(sp + 0xFFFE) % 0x1_0000]
                            && m[(sp + 0xFFFE) % 0x1_0000] < 0x100)
                ),
        )]
        contract_push_af: {
            rom_end,
            k: [Instruction::Push { pair: Pair16::Af }],
            regs: r,
            f: Flags { z: zero, n: subtract, h: half_carry, c: carry },
            sp: sp,
            memory: m,
            bus: log,
        } => {
            k: [],
            sp: ((sp + 0xFFFE) % 0x1_0000),
            memory: k::map![
                ((sp + 0xFFFF) % 0x1_0000) -> r[Register::A],
                ((sp + 0xFFFE) % 0x1_0000) -> ExpectedFlagByte(zero, subtract, half_carry, carry),
                ..m,
            ],
            bus: [
                ..log,
                Event::Idle,
                Event::Write {
                    addr: ((sp + 0xFFFF) % 0x1_0000),
                    value: r[Register::A],
                },
                Event::Write {
                    addr: ((sp + 0xFFFE) % 0x1_0000),
                    value: ExpectedFlagByte(zero, subtract, half_carry, carry),
                },
            ],
        };

        /// Stack transfer: wrapping SP, hardware byte order, preserved state; POP AF masks the low flag nibble.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= sp && sp < 0x1_0000)
                        && (0 <= m[sp] && m[sp] < 0x100)
                        && (0 <= m[(sp + 1) % 0x1_0000] && m[(sp + 1) % 0x1_0000] < 0x100)
                ),
        )]
        contract_pop_registers: {
            rom_end,
            k: [Instruction::Pop { pair: Pair16::Registers { high, low } }],
            regs: r,
            sp: sp,
            memory: m,
            bus: log,
            w: _,
            z: _,
        } => {
            k: [],
            sp: ((sp + 2) % 0x1_0000),
            w: m[(sp + 1) % 0x1_0000],
            z: m[sp],
            bus: [
                ..log,
                Event::Read { addr: sp, value: m[sp] },
                Event::Read { addr: ((sp + 1) % 0x1_0000), value: m[(sp + 1) % 0x1_0000] },
            ],
            regs: k::map![high -> m[(sp + 1) % 0x1_0000], low -> m[sp], ..r],
        };

        /// Stack transfer: wrapping SP, hardware byte order, preserved state; POP AF masks the low flag nibble.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= sp && sp < 0x1_0000)
                        && (0 <= m[sp] && m[sp] < 0x100)
                        && (0 <= m[(sp + 1) % 0x1_0000] && m[(sp + 1) % 0x1_0000] < 0x100)
                ),
        )]
        contract_pop_af: {
            rom_end,
            k: [Instruction::Pop { pair: Pair16::Af }],
            regs: r,
            f: _,
            sp: sp,
            memory: m,
            bus: log,
            w: _,
            z: _,
        } => {
            k: [],
            sp: ((sp + 2) % 0x1_0000),
            w: m[(sp + 1) % 0x1_0000],
            z: m[sp],
            bus: [
                ..log,
                Event::Read { addr: sp, value: m[sp] },
                Event::Read { addr: ((sp + 1) % 0x1_0000), value: m[(sp + 1) % 0x1_0000] },
            ],
            regs: k::map![Register::A -> m[(sp + 1) % 0x1_0000], ..r],
            f: Flags {
                z: m[sp] / 0x80 == 1,
                n: (m[sp] / 0x40) % 2 == 1,
                h: (m[sp] / 0x20) % 2 == 1,
                c: (m[sp] / 0x10) % 2 == 1,
            },
        };

        /// Signed SP addition: sign extension for the result, unsigned low-byte carry flags, Z=N=0.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= pc && pc < 0x1_0000)
                        && (0 <= sp && sp < 0x1_0000)
                        && (0 <= m[pc] && m[pc] < 0x100)
                ),
        )]
        contract_sp_offset_hl: {
            rom_end,
            k: [Instruction::LdHlSpOffset],
            pc: pc,
            sp: sp,
            memory: m,
            f: _,
            z: _,
            bus: log,
            regs: r,
        } => {
            k: [],
            pc: ((pc + 1) % 0x1_0000),
            f: Flags {
                z: false,
                n: false,
                h: sp % 0x10 + m[pc] % 0x10 >= 0x10,
                c: sp % 0x100 + m[pc] >= 0x100,
            },
            z: m[pc],
            bus: [..log, Event::Read { addr: pc, value: m[pc] }, Event::Idle],
            regs: k::map![
                Register::H -> int::rem_euclid(
                    sp + kequal::ite(m[pc] < 0x80, m[pc], m[pc] - 0x100),
                    0x1_0000,
                ) / 0x100,
                Register::L -> int::rem_euclid(
                    sp + kequal::ite(m[pc] < 0x80, m[pc], m[pc] - 0x100),
                    0x1_0000,
                ) % 0x100,
                ..r,
            ],
        };

        /// Signed SP addition: sign extension for the result, unsigned low-byte carry flags, Z=N=0.
        #[k::claim(all_paths)]
        #[k::requires(
            rom_end == 0
                && (
                    (0 <= pc && pc < 0x1_0000)
                        && (0 <= sp && sp < 0x1_0000)
                        && (0 <= m[pc] && m[pc] < 0x100)
                ),
        )]
        contract_sp_offset_sp: {
            rom_end,
            k: [Instruction::AddSpOffset],
            pc: pc,
            sp: sp,
            memory: m,
            f: _,
            z: _,
            bus: log,
        } => {
            k: [],
            pc: ((pc + 1) % 0x1_0000),
            f: Flags {
                z: false,
                n: false,
                h: sp % 0x10 + m[pc] % 0x10 >= 0x10,
                c: sp % 0x100 + m[pc] >= 0x100,
            },
            z: m[pc],
            bus: [..log, Event::Read { addr: pc, value: m[pc] }, Event::Idle, Event::Idle],
            sp: int::rem_euclid(sp + kequal::ite(m[pc] < 0x80, m[pc], m[pc] - 0x100), 0x1_0000),
        };
    }
}
