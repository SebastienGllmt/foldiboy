//! Micro-operations: the register transfers and bus cycles an instruction
//! is made of.
//!
//! A bus micro-operation (`Fetch`, `FetchCb`, `ReadZ`, `ReadW`,
//! `WriteByte`, `Idle`) takes one machine cycle and appends one entry to
//! the bus log; every other micro-operation takes no time.

use k::builtin::{Int, bytes};

use crate::alu::{AluOp, UnaryOp};
use crate::decode::{AddrMode, Address, Cond, Condition, Operand, Operand8, Pair16, RegPair};
use crate::machine::{Event, Latch, Register};
use crate::{
    AluFlags, AluResult, BitFlags, ByteFlags, Decode, DecodeCb, High, Hl, Holds, InRom, Low,
    Machine, Pair, Signed, SpOffsetFlags, UnaryFlags, UnaryResult, Word,
};

k::sorts! {
    MicroOp: k::KItem;
}

#[k::constructors(sort = MicroOp, result = k::KResult)]
pub enum Micro {
    /// Read the opcode at `PC`, advance `PC`, and decode it.
    #[k::syntax("#fetch")]
    Fetch,
    /// Read the opcode after the `0xCB` prefix, advance `PC`, and decode it.
    #[k::syntax("#fetchCB")]
    FetchCb,
    /// `Z <- src`.
    #[k::syntax("#loadZ({src})")]
    LoadZ { src: Operand8 },
    /// `W <- src`.
    #[k::syntax("#loadW({src})")]
    LoadW { src: Operand8 },
    /// `dst <- Z`.
    #[k::syntax("#storeZ({dst})")]
    StoreZ { dst: Operand8 },
    /// `pair <- WZ`.
    #[k::syntax("#storePair({pair})")]
    StorePair { pair: RegPair },
    /// `pair <- value`.
    #[k::syntax("#setPair({pair}, {value})")]
    SetPair { pair: RegPair, value: Int },
    /// `WZ <-` the address `mode` names, reading any immediate bytes.
    #[k::syntax("#resolveAddress({mode})")]
    ResolveAddress { mode: AddrMode },
    /// `Z <- [WZ]`.
    #[k::syntax("#readZAtWZ")]
    ReadZAtWz,
    /// `[WZ] <- A`.
    #[k::syntax("#writeAAtWZ")]
    WriteAAtWz,
    /// `[WZ] <- low SP`, then `[WZ + 1] <- high SP`.
    #[k::syntax("#storeSpAtWZ")]
    StoreSpAtWz,
    /// `Z <- [addr]`.
    #[k::syntax("#readZ({addr})")]
    ReadZ { addr: Int },
    /// `W <- [addr]`.
    #[k::syntax("#readW({addr})")]
    ReadW { addr: Int },
    /// `[addr] <- value`.
    #[k::syntax("#writeByte({addr}, {value})")]
    WriteByte { addr: Int, value: Int },
    /// A machine cycle without a bus access.
    #[k::syntax("#idle")]
    Idle,
    /// `A <- A op Z`.
    #[k::syntax("#aluA({op})")]
    AluA { op: AluOp },
    /// `Z <- op Z`.
    #[k::syntax("#updateZ({op})")]
    UpdateZ { op: UnaryOp },
    /// Set the flags from bit `bit` of `Z`.
    #[k::syntax("#checkBit({bit})")]
    CheckBit { bit: Int },
    /// `dst <- SP + signed Z`, with the flags of the low-byte addition.
    #[k::syntax("#addSpZ({dst})")]
    AddSpZ { dst: RegPair },
    /// If `cond` holds: `PC <- PC + signed Z`, taking one more cycle.
    #[k::syntax("#jumpRelativeIf({cond})")]
    JumpRelativeIf { cond: Cond },
    /// If `cond` holds: `PC <- WZ`, taking one more cycle.
    #[k::syntax("#jumpIf({cond})")]
    JumpIf { cond: Cond },
    /// If `cond` holds: push `PC` and `PC <- WZ`.
    #[k::syntax("#callIf({cond})")]
    CallIf { cond: Cond },
    /// If `cond` holds: pop `PC`.
    #[k::syntax("#popPcIf({cond})")]
    PopPcIf { cond: Cond },
}

k::rules! {
    for Machine;

    #[k::requires(!bug && !InRom(rom_end, pc))]
    fetch: {
        k: [Micro::Fetch, ..],
        pc,
        halt_bug: bug,
        rom_end,
        memory: { pc -> opcode, .. },
        bus: log,
    } => {
        k: [Decode(opcode), ..],
        pc: Word(pc + 1),
        bus: [..log, Event::Read { addr: pc, value: opcode }],
    };

    /// As `fetch`, reading the ROM.
    #[k::requires(!bug && InRom(rom_end, pc))]
    fetch_rom: {
        k: [Micro::Fetch, ..],
        pc,
        halt_bug: bug,
        rom,
        rom_end,
        bus: log,
    } => {
        k: [Decode(bytes::get(rom, pc)), ..],
        pc: Word(pc + 1),
        bus: [..log, Event::Read { addr: pc, value: bytes::get(rom, pc) }],
    };

    /// After the `HALT` bug, the fetch leaves `PC` on the opcode it read.
    #[k::requires(bug && !InRom(rom_end, pc))]
    fetch_halt_bug: {
        k: [Micro::Fetch, ..],
        pc,
        halt_bug: bug,
        rom_end,
        memory: { pc -> opcode, .. },
        bus: log,
    } => {
        k: [Decode(opcode), ..],
        halt_bug: false,
        bus: [..log, Event::Read { addr: pc, value: opcode }],
    };

    /// As `fetch_halt_bug`, reading the ROM.
    #[k::requires(bug && InRom(rom_end, pc))]
    fetch_halt_bug_rom: {
        k: [Micro::Fetch, ..],
        pc,
        halt_bug: bug,
        rom,
        rom_end,
        bus: log,
    } => {
        k: [Decode(bytes::get(rom, pc)), ..],
        halt_bug: false,
        bus: [..log, Event::Read { addr: pc, value: bytes::get(rom, pc) }],
    };

    #[k::requires(!InRom(rom_end, pc))]
    fetch_cb: {
        k: [Micro::FetchCb, ..],
        pc,
        rom_end,
        memory: { pc -> opcode, .. },
        bus: log,
    } => {
        k: [DecodeCb(opcode), ..],
        pc: Word(pc + 1),
        bus: [..log, Event::Read { addr: pc, value: opcode }],
    };

    /// As `fetch_cb`, reading the ROM.
    #[k::requires(InRom(rom_end, pc))]
    fetch_cb_rom: {
        k: [Micro::FetchCb, ..],
        pc,
        rom,
        rom_end,
        bus: log,
    } => {
        k: [DecodeCb(bytes::get(rom, pc)), ..],
        pc: Word(pc + 1),
        bus: [..log, Event::Read { addr: pc, value: bytes::get(rom, pc) }],
    };

    load_z_register: {
        k: [Micro::LoadZ { src: Operand::Reg { r } }, ..],
        regs: { r -> value, .. },
        z: _,
    } => {
        k: [..],
        z: value,
    };

    load_z_hl_indirect: {
        k: [Micro::LoadZ { src: Operand::HlIndirect }, ..],
        regs: registers,
    } => {
        k: [Micro::ReadZ { addr: Hl(registers) }, ..],
    };

    load_z_immediate: {
        k: [Micro::LoadZ { src: Operand::Immediate }, ..],
        pc,
    } => {
        k: [Micro::ReadZ { addr: pc }, ..],
        pc: Word(pc + 1),
    };

    load_w_immediate: {
        k: [Micro::LoadW { src: Operand::Immediate }, ..],
        pc,
    } => {
        k: [Micro::ReadW { addr: pc }, ..],
        pc: Word(pc + 1),
    };

    store_z_register: {
        k: [Micro::StoreZ { dst: Operand::Reg { r } }, ..],
        regs: registers,
        z: value,
    } => {
        k: [..],
        regs: k::map![r -> value, ..registers],
    };

    store_z_hl_indirect: {
        k: [Micro::StoreZ { dst: Operand::HlIndirect }, ..],
        regs: registers,
        z: value,
    } => {
        k: [Micro::WriteByte { addr: Hl(registers), value }, ..],
    };

    store_pair: {
        k: [Micro::StorePair { pair }, ..],
        w,
        z,
    } => {
        k: [Micro::SetPair { pair, value: Pair(w, z) }, ..],
    };

    set_pair_registers: {
        k: [Micro::SetPair { pair: Pair16::Registers { high, low }, value }, ..],
        regs: registers,
    } => {
        k: [..],
        regs: k::map![high -> High(value), low -> Low(value), ..registers],
    };

    /// `F` takes the flags of the low byte, whose bits 3 to 0 it has no place for.
    set_pair_af: {
        k: [Micro::SetPair { pair: Pair16::Af, value }, ..],
        regs: registers,
        f: _,
    } => {
        k: [..],
        regs: k::map![Register::A -> High(value), ..registers],
        f: ByteFlags(Low(value)),
    };

    set_pair_sp: {
        k: [Micro::SetPair { pair: Pair16::StackPointer, value }, ..],
        sp: _,
    } => {
        k: [..],
        sp: value,
    };

    resolve_via_pair: {
        k: [Micro::ResolveAddress { mode: Address::ViaPair { high, low } }, ..],
        regs: registers,
        w: _,
        z: _,
    } => {
        k: [..],
        w: registers[high],
        z: registers[low],
    };

    resolve_via_hl_step: {
        k: [Micro::ResolveAddress { mode: Address::ViaHlStep { delta } }, ..],
        regs: registers,
        w: _,
        z: _,
    } => {
        k: [
            Micro::SetPair {
                pair: Pair16::Registers { high: Register::H, low: Register::L },
                value: Word(Hl(registers) + delta),
            },
            ..,
        ],
        regs: registers,
        w: registers[Register::H],
        z: registers[Register::L],
    };

    resolve_high_immediate: {
        k: [Micro::ResolveAddress { mode: Address::HighImmediate }, ..],
        w: _,
    } => {
        k: [Micro::LoadZ { src: Operand::Immediate }, ..],
        w: 0xFF,
    };

    resolve_high_c: {
        k: [Micro::ResolveAddress { mode: Address::HighC }, ..],
        regs: { Register::C -> c, .. },
        w: _,
        z: _,
    } => {
        k: [..],
        w: 0xFF,
        z: c,
    };

    resolve_absolute: {
        k: [Micro::ResolveAddress { mode: Address::Absolute }, ..],
    } => {
        k: [Micro::LoadZ { src: Operand::Immediate }, Micro::LoadW { src: Operand::Immediate }, ..],
    };

    read_z_at_wz: {
        k: [Micro::ReadZAtWz, ..],
        w,
        z,
    } => {
        k: [Micro::ReadZ { addr: Pair(w, z) }, ..],
    };

    write_a_at_wz: {
        k: [Micro::WriteAAtWz, ..],
        regs: { Register::A -> a, .. },
        w,
        z,
    } => {
        k: [Micro::WriteByte { addr: Pair(w, z), value: a }, ..],
    };

    store_sp_at_wz: {
        k: [Micro::StoreSpAtWz, ..],
        sp,
        w,
        z,
    } => {
        k: [
            Micro::WriteByte { addr: Pair(w, z), value: Low(sp) },
            Micro::WriteByte { addr: Word(Pair(w, z) + 1), value: High(sp) },
            ..,
        ],
    };

    #[k::requires(!InRom(rom_end, addr))]
    read_z: {
        k: [Micro::ReadZ { addr }, ..],
        rom_end,
        memory: { addr -> value, .. },
        z: _,
        bus: log,
    } => {
        k: [..],
        z: value,
        bus: [..log, Event::Read { addr, value }],
    };

    /// As `read_z`, reading the ROM.
    #[k::requires(InRom(rom_end, addr))]
    read_z_rom: {
        k: [Micro::ReadZ { addr }, ..],
        rom,
        rom_end,
        z: _,
        bus: log,
    } => {
        k: [..],
        z: bytes::get(rom, addr),
        bus: [..log, Event::Read { addr, value: bytes::get(rom, addr) }],
    };

    #[k::requires(!InRom(rom_end, addr))]
    read_w: {
        k: [Micro::ReadW { addr }, ..],
        rom_end,
        memory: { addr -> value, .. },
        w: _,
        bus: log,
    } => {
        k: [..],
        w: value,
        bus: [..log, Event::Read { addr, value }],
    };

    /// As `read_w`, reading the ROM.
    #[k::requires(InRom(rom_end, addr))]
    read_w_rom: {
        k: [Micro::ReadW { addr }, ..],
        rom,
        rom_end,
        w: _,
        bus: log,
    } => {
        k: [..],
        w: bytes::get(rom, addr),
        bus: [..log, Event::Read { addr, value: bytes::get(rom, addr) }],
    };

    /// A write outside the ROM stores the byte, at an address the map holds or not.
    #[k::requires(!InRom(rom_end, addr))]
    write_byte: {
        k: [Micro::WriteByte { addr, value }, ..],
        rom_end,
        memory,
        bus: log,
    } => {
        k: [..],
        memory: k::map![addr -> Latch(addr, value), ..memory],
        bus: [..log, Event::Write { addr, value }],
    };

    /// A write to the ROM changes nothing a cartridge without a mapper holds; the
    /// cycle still drives the bus.
    #[k::requires(InRom(rom_end, addr))]
    write_byte_rom: {
        k: [Micro::WriteByte { addr, value }, ..],
        rom_end,
        bus: log,
    } => {
        k: [..],
        bus: [..log, Event::Write { addr, value }],
    };

    idle: {
        k: [Micro::Idle, ..],
        bus: log,
    } => {
        k: [..],
        bus: [..log, Event::Idle],
    };

    alu_a: {
        k: [Micro::AluA { op }, ..],
        regs: registers,
        z: value,
        f: flags,
    } => {
        k: [..],
        regs: k::map![
            Register::A -> AluResult(op, registers[Register::A], value, flags),
            ..registers,
        ],
        f: AluFlags(op, registers[Register::A], value, flags),
    };

    update_z: {
        k: [Micro::UpdateZ { op }, ..],
        z: value,
        f: flags,
    } => {
        k: [..],
        z: UnaryResult(op, value, flags),
        f: UnaryFlags(op, value, flags),
    };

    check_bit: {
        k: [Micro::CheckBit { bit }, ..],
        z: value,
        f: flags,
    } => {
        k: [..],
        f: BitFlags(bit, value, flags),
    };

    #[k::requires(Holds(cond, flags))]
    jump_relative_taken: {
        k: [Micro::JumpRelativeIf { cond }, ..],
        pc,
        z: offset,
        f: flags,
    } => {
        k: [Micro::Idle, ..],
        pc: Word(pc + Signed(offset)),
    };

    #[k::requires(!Holds(cond, flags))]
    jump_relative_not_taken: {
        k: [Micro::JumpRelativeIf { cond }, ..],
        f: flags,
    } => {
        k: [..],
    };

    #[k::requires(Holds(cond, flags))]
    jump_taken: {
        k: [Micro::JumpIf { cond }, ..],
        pc: _,
        w: high,
        z: low,
        f: flags,
    } => {
        k: [Micro::Idle, ..],
        pc: Pair(high, low),
    };

    #[k::requires(!Holds(cond, flags))]
    jump_not_taken: {
        k: [Micro::JumpIf { cond }, ..],
        f: flags,
    } => {
        k: [..],
    };

    add_sp_z: {
        k: [Micro::AddSpZ { dst }, ..],
        sp,
        z: offset,
        f: _,
    } => {
        k: [Micro::SetPair { pair: dst, value: Word(sp + Signed(offset)) }, ..],
        f: SpOffsetFlags(sp, offset),
    };

    /// An internal cycle, then the return address high byte and low byte.
    #[k::requires(Holds(cond, flags))]
    call_taken: {
        k: [Micro::CallIf { cond }, ..],
        pc,
        sp,
        w: high,
        z: low,
        f: flags,
    } => {
        k: [
            Micro::Idle,
            Micro::WriteByte { addr: Word(sp - 1), value: High(pc) },
            Micro::WriteByte { addr: Word(sp - 2), value: Low(pc) },
            ..,
        ],
        pc: Pair(high, low),
        sp: Word(sp - 2),
    };

    #[k::requires(!Holds(cond, flags))]
    call_not_taken: {
        k: [Micro::CallIf { cond }, ..],
        f: flags,
    } => {
        k: [..],
    };

    /// The low byte, the high byte, then an internal cycle setting `PC`.
    #[k::requires(Holds(cond, flags))]
    pop_pc_taken: {
        k: [Micro::PopPcIf { cond }, ..],
        sp,
        f: flags,
    } => {
        k: [
            Micro::ReadZ { addr: sp },
            Micro::ReadW { addr: Word(sp + 1) },
            Micro::JumpIf { cond: Condition::Always },
            ..,
        ],
        sp: Word(sp + 2),
    };

    #[k::requires(!Holds(cond, flags))]
    pop_pc_not_taken: {
        k: [Micro::PopPcIf { cond }, ..],
        f: flags,
    } => {
        k: [..],
    };
}
