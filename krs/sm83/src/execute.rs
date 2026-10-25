//! Each decoded instruction as its sequence of micro-operations.

use crate::decode::{Address, Condition, Instruction, Operand, Pair16};
use crate::machine::Register;
use crate::micro::Micro;
use crate::{AddHlFlags, High, Hl, InterruptRequested, Low, Machine, PairValue, Word};

k::rules! {
    for Machine;

    nop: {
        k: [Instruction::Nop, ..],
    } => {
        k: [..],
    };

    ld8: {
        k: [Instruction::Ld8 { dst, src }, ..],
    } => {
        k: [Micro::LoadZ { src }, Micro::StoreZ { dst }, ..],
    };

    alu8: {
        k: [Instruction::Alu8 { op, src }, ..],
    } => {
        k: [Micro::LoadZ { src }, Micro::AluA { op }, ..],
    };

    update8: {
        k: [Instruction::Update8 { op, target }, ..],
    } => {
        k: [Micro::LoadZ { src: target }, Micro::UpdateZ { op }, Micro::StoreZ { dst: target }, ..],
    };

    test_bit: {
        k: [Instruction::TestBit { bit, target }, ..],
    } => {
        k: [Micro::LoadZ { src: target }, Micro::CheckBit { bit }, ..],
    };

    jump_relative: {
        k: [Instruction::JumpRelative { cond }, ..],
    } => {
        k: [Micro::LoadZ { src: Operand::Immediate }, Micro::JumpRelativeIf { cond }, ..],
    };

    jump: {
        k: [Instruction::Jump { cond }, ..],
    } => {
        k: [
            Micro::LoadZ { src: Operand::Immediate },
            Micro::LoadW { src: Operand::Immediate },
            Micro::JumpIf { cond },
            ..
        ],
    };

    /// An internal cycle, then the high byte and the low byte, each below
    /// the stack pointer.
    push: {
        k: [Instruction::Push { pair }, ..],
        regs: registers,
        f: flags,
        sp,
    } => {
        k: [
            Micro::Idle,
            Micro::WriteByte {
                addr: Word(sp - 1),
                value: High(PairValue(pair, registers, flags, sp)),
            },
            Micro::WriteByte {
                addr: Word(sp - 2),
                value: Low(PairValue(pair, registers, flags, sp)),
            },
            ..
        ],
        sp: Word(sp - 2),
    };

    /// The low byte, then the high byte, each at the stack pointer.
    pop: {
        k: [Instruction::Pop { pair }, ..],
        sp,
    } => {
        k: [
            Micro::ReadZ { addr: sp },
            Micro::ReadW { addr: Word(sp + 1) },
            Micro::StorePair { pair },
            ..
        ],
        sp: Word(sp + 2),
    };

    prefix: {
        k: [Instruction::Prefix, ..],
    } => {
        k: [Micro::FetchCb, ..],
    };

    load_a: {
        k: [Instruction::LoadA { addr }, ..],
    } => {
        k: [
            Micro::ResolveAddress { mode: addr },
            Micro::ReadZAtWz,
            Micro::StoreZ { dst: Operand::Reg { r: Register::A } },
            ..
        ],
    };

    store_a: {
        k: [Instruction::StoreA { addr }, ..],
    } => {
        k: [Micro::ResolveAddress { mode: addr }, Micro::WriteAAtWz, ..],
    };

    ld16: {
        k: [Instruction::Ld16 { pair }, ..],
    } => {
        k: [
            Micro::LoadZ { src: Operand::Immediate },
            Micro::LoadW { src: Operand::Immediate },
            Micro::StorePair { pair },
            ..
        ],
    };

    store_sp: {
        k: [Instruction::StoreSp, ..],
    } => {
        k: [Micro::ResolveAddress { mode: Address::Absolute }, Micro::StoreSpAtWz, ..],
    };

    ld_sp_hl: {
        k: [Instruction::LdSpHl, ..],
        regs: registers,
        sp: _,
    } => {
        k: [Micro::Idle, ..],
        sp: Hl(registers),
    };

    ld_hl_sp_offset: {
        k: [Instruction::LdHlSpOffset, ..],
    } => {
        k: [
            Micro::LoadZ { src: Operand::Immediate },
            Micro::Idle,
            Micro::AddSpZ { dst: Pair16::Registers { high: Register::H, low: Register::L } },
            ..
        ],
    };

    add_sp_offset: {
        k: [Instruction::AddSpOffset, ..],
    } => {
        k: [
            Micro::LoadZ { src: Operand::Immediate },
            Micro::Idle,
            Micro::Idle,
            Micro::AddSpZ { dst: Pair16::StackPointer },
            ..
        ],
    };

    inc16: {
        k: [Instruction::Inc16 { pair }, ..],
        regs: registers,
        f: flags,
        sp,
    } => {
        k: [
            Micro::Idle,
            Micro::SetPair {
                pair,
                value: Word(PairValue(pair, registers, flags, sp) + 1),
            },
            ..
        ],
    };

    dec16: {
        k: [Instruction::Dec16 { pair }, ..],
        regs: registers,
        f: flags,
        sp,
    } => {
        k: [
            Micro::Idle,
            Micro::SetPair {
                pair,
                value: Word(PairValue(pair, registers, flags, sp) - 1),
            },
            ..
        ],
    };

    add_hl: {
        k: [Instruction::AddHl { pair }, ..],
        regs: registers,
        f: flags,
        sp,
    } => {
        k: [
            Micro::Idle,
            Micro::SetPair {
                pair: Pair16::Registers { high: Register::H, low: Register::L },
                value: Word(Hl(registers) + PairValue(pair, registers, flags, sp)),
            },
            ..
        ],
        f: AddHlFlags(Hl(registers), PairValue(pair, registers, flags, sp), flags),
    };

    jump_hl: {
        k: [Instruction::JumpHl, ..],
        regs: registers,
        pc: _,
    } => {
        k: [..],
        pc: Hl(registers),
    };

    call: {
        k: [Instruction::Call { cond }, ..],
    } => {
        k: [
            Micro::LoadZ { src: Operand::Immediate },
            Micro::LoadW { src: Operand::Immediate },
            Micro::CallIf { cond },
            ..
        ],
    };

    return_always: {
        k: [Instruction::Return, ..],
    } => {
        k: [Micro::PopPcIf { cond: Condition::Always }, ..],
    };

    return_if: {
        k: [Instruction::ReturnIf { cond }, ..],
    } => {
        k: [Micro::Idle, Micro::PopPcIf { cond }, ..],
    };

    /// `RETI` enables interrupts without the delay of `EI`.
    return_from_interrupt: {
        k: [Instruction::ReturnFromInterrupt, ..],
        ime: _,
    } => {
        k: [Micro::PopPcIf { cond: Condition::Always }, ..],
        ime: true,
    };

    /// An internal cycle, then the return address high byte and low byte.
    restart: {
        k: [Instruction::Restart { target }, ..],
        pc,
        sp,
    } => {
        k: [
            Micro::Idle,
            Micro::WriteByte { addr: Word(sp - 1), value: High(pc) },
            Micro::WriteByte { addr: Word(sp - 2), value: Low(pc) },
            ..
        ],
        pc: target,
        sp: Word(sp - 2),
    };

    disable_interrupts: {
        k: [Instruction::DisableInterrupts, ..],
        ime: _,
        ei_pending: _,
    } => {
        k: [..],
        ime: false,
        ei_pending: false,
    };

    /// `EI` takes effect after the following instruction; see the boundary
    /// rules in `interrupt.rs`.
    enable_interrupts: {
        k: [Instruction::EnableInterrupts, ..],
        ei_pending: _,
    } => {
        k: [..],
        ei_pending: true,
    };

    /// With `IME` set and an interrupt already requested, `HALT` does not
    /// halt and leaves `PC` on itself, so the handler returns to the `HALT`.
    #[k::requires(ime && InterruptRequested(memory))]
    halt_dispatching: {
        k: [Instruction::Halt, ..],
        pc,
        ime,
        memory,
    } => {
        k: [..],
        pc: Word(pc - 1),
    };

    /// The `HALT` bug: with `IME` clear and an interrupt already requested,
    /// `HALT` does not halt and the next opcode byte is fetched twice.
    #[k::requires(!ime && InterruptRequested(memory))]
    halt_bug: {
        k: [Instruction::Halt, ..],
        ime,
        halt_bug: _,
        memory,
    } => {
        k: [..],
        halt_bug: true,
    };

    #[k::requires(!InterruptRequested(memory))]
    halt: {
        k: [Instruction::Halt, ..],
        halted: _,
        memory,
    } => {
        k: [..],
        halted: true,
    };
}
