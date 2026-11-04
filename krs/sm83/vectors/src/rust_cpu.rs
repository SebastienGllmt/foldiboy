//! The Rust emulator's CPU (`lib/crates/cpu`) as a corpus system.
//!
//! The CPU runs over the CPU stage's memory model, the one the SM83 definition states, so a
//! comparison is between the two CPUs and not between two memory maps: the ROM region is
//! read-only, every other byte holds only what the initial state or a write put there, and a
//! read of any other byte has no value, which ends the run as `Stuck`, as it does in the
//! definition. The initial state is the definition's (`InitialState::from_krs`).
//!
//! What the adapter decides, because the Rust CPU does not:
//! - An unused opcode locks: its fetch is logged and `PC` advances as the definition's fetch
//!   does, then the run ends `Locked` (the Rust CPU treats it as a NOP).
//! - `STOP` is not covered (the definition is stuck on it): its fetch is logged and the run
//!   ends `Stuck`.
//! - The interrupt logic's `IE`/`IF` accesses are not bus cycles
//!   ([`memory::Bus::read_interrupt_register`]) and are not logged.
//!
//! The Rust CPU executes each instruction's accesses at once, without the internal cycles, so
//! its log has no `idle` events and reads may be ordered differently: compare it at the
//! Writes tier.

use std::collections::BTreeMap;

use cpu::GbCpu;
use krs_exec::StateView;
use memory::Bus;

use crate::observation::{EndClass, EndState, Observation};
use crate::{Access, Names, cell, int as cell_int};

/// The opcodes no instruction uses (Pan Docs): executing one locks the CPU.
const UNUSED: [u8; 11] = [0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD];
const STOP: u8 = 0x10;

/// A case's initial state, as the SM83 definition's initializer builds it.
#[derive(Clone, Debug)]
pub struct InitialState {
    pub state: EndState,
    /// The ROM region's bytes, readable at `0..rom.len()`.
    pub rom: Vec<u8>,
}

impl InitialState {
    /// The state of a configuration of the SM83 definition before its first step.
    pub fn from_krs(names: &Names, view: &StateView) -> Result<Self, String> {
        let rom = cell(view, "rom")?.as_bytes().ok_or("<rom> is not Bytes")?.to_vec();
        let rom_end = cell_int(view, "rom_end")?;
        if usize::try_from(rom_end).ok() != Some(rom.len()) {
            return Err(format!("rom_end {rom_end} is not the ROM's length {}", rom.len()));
        }
        Ok(Self { state: EndState::from_krs(names, view)?, rom })
    }
}

/// The CPU stage's memory, recording each bus access as an event.
struct CorpusBus {
    rom: Vec<u8>,
    memory: BTreeMap<u16, u8>,
    log: Vec<Access>,
    /// The first address read that holds no byte: the run is stuck there.
    undefined: Option<u16>,
}

impl CorpusBus {
    fn byte(&self, addr: u16) -> Option<u8> {
        match self.rom.get(usize::from(addr)) {
            Some(&byte) => Some(byte),
            None => self.memory.get(&addr).copied(),
        }
    }

    fn load(&mut self, addr: u16) -> u8 {
        self.byte(addr).unwrap_or_else(|| {
            self.undefined.get_or_insert(addr);
            0xFF
        })
    }
}

impl Bus for CorpusBus {
    fn read(&mut self, addr: u16) -> u8 {
        let value = self.load(addr);
        self.log.push(Access::Read { addr: addr.into(), value: value.into() });
        value
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.log.push(Access::Write { addr: addr.into(), value: value.into() });
        if usize::from(addr) >= self.rom.len() {
            self.memory.insert(addr, value);
        }
    }

    fn read_interrupt_register(&mut self, addr: u16) -> u8 {
        self.load(addr)
    }

    fn write_interrupt_register(&mut self, addr: u16, value: u8) {
        self.memory.insert(addr, value);
    }
}

/// What a run of the Rust CPU cost: instructions executed and M-cycles.
#[derive(Clone, Copy, Debug, Default)]
pub struct Cost {
    pub instructions: u64,
    pub cycles: u64,
}

fn byte(value: i64, what: &str) -> Result<u8, String> {
    u8::try_from(value).map_err(|_| format!("{what} {value} is not a byte"))
}

fn word(value: i64, what: &str) -> Result<u16, String> {
    u16::try_from(value).map_err(|_| format!("{what} {value} is not a word"))
}

/// Runs the Rust CPU from `initial` until it locks, gets stuck, or has executed `budget`
/// instructions.
pub fn run(initial: &InitialState, budget: u64) -> Result<(Observation, Cost), String> {
    let s = &initial.state;
    let mut memory = BTreeMap::new();
    for (&addr, &value) in &s.memory {
        memory.insert(word(addr, "address")?, byte(value, "memory byte")?);
    }
    let bus = CorpusBus { rom: initial.rom.clone(), memory, log: Vec::new(), undefined: None };
    let mut cpu = GbCpu::new(bus);
    let [a, b, c, d, e, h, l] = s.registers;
    cpu.regs.a = byte(a, "A")?;
    cpu.regs.b = byte(b, "B")?;
    cpu.regs.c = byte(c, "C")?;
    cpu.regs.d = byte(d, "D")?;
    cpu.regs.e = byte(e, "E")?;
    cpu.regs.h = byte(h, "H")?;
    cpu.regs.l = byte(l, "L")?;
    let [zf, nf, hf, cf] = s.flags;
    cpu.regs.f = (u8::from(zf) << 7) | (u8::from(nf) << 6) | (u8::from(hf) << 5) | (u8::from(cf) << 4);
    cpu.regs.pc = word(s.pc, "PC")?;
    cpu.regs.sp = word(s.sp, "SP")?;
    cpu.ime = s.ime;
    // After EI the definition's boundary holds eiPending; the Rust CPU's counter is at 0 there.
    cpu.ime_delay = s.ei_pending.then_some(0);
    cpu.halted = s.halted;
    cpu.halt_bug = s.halt_bug;

    let mut cost = Cost::default();
    let end = loop {
        if cpu.bus.undefined.is_some() {
            break EndClass::Stuck;
        }
        if !cpu.halted {
            let pc = cpu.regs.pc;
            if let Some(opcode) = cpu.bus.byte(pc).filter(|op| UNUSED.contains(op) || *op == STOP) {
                cpu.bus.read(pc);
                if !cpu.halt_bug {
                    cpu.regs.pc = pc.wrapping_add(1);
                }
                cpu.halt_bug = false;
                break if opcode == STOP { EndClass::Stuck } else { EndClass::Locked };
            }
        }
        if cost.instructions == budget {
            break EndClass::Bound;
        }
        cost.cycles += u64::from(cpu.step());
        cost.instructions += 1;
    };

    let r = &cpu.regs;
    let state = EndState {
        registers: [r.a, r.b, r.c, r.d, r.e, r.h, r.l].map(i64::from),
        flags: [r.zf(), r.nf(), r.hf(), r.cf()],
        pc: r.pc.into(),
        sp: r.sp.into(),
        ime: cpu.ime,
        ei_pending: cpu.ime_delay.is_some(),
        halted: cpu.halted,
        halt_bug: cpu.halt_bug,
        memory: cpu.bus.memory.iter().map(|(&addr, &value)| (addr.into(), value.into())).collect(),
    };
    Ok((Observation { end, log: std::mem::take(&mut cpu.bus.log), state }, cost))
}
