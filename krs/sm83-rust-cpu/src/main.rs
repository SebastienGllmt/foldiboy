//! The Rust emulator's CPU (`lib/crates/cpu`) as an implementation of the SM83 definition that
//! `krs kink compare` compares with it on every corpus program (corpus/FLOW.toml
//! `[[implementation]]`), by the protocol `kink-implementation/1` (k-rust's kink-flow README,
//! "Comparing implementations"):
//!
//!     sm83-rust-cpu --kink < REQUEST.json > ANSWER.json
//!
//! The request's `start` is the program's initial state as the definition builds it (its
//! initializer calls the specification's boot functions), every cell by Rust field path. The
//! answer is `{"end": "stopped" | "final", "cells": ..}`: that start with the cells the CPU changes
//! written over it, in the same encoding, leaving out `k`, `w` and `z` (FLOW.toml's `omits`),
//! or `{"end": "could-not-apply", "reason": ..}`. A run that ends neither way exits 1.
//!
//! The CPU runs over the CPU stage's memory model, the one the definition states, so a
//! comparison is between the two CPUs and not between two memory maps: the ROM region is
//! read-only, every other byte holds only what the initial state or a write put there, and a
//! read of any other byte has no value, which ends the run stuck, as it does in the definition.
//! `IF`'s bits 7 to 5 read as 1: a write to it stores them set, as the definition's `Latch` does.
//!
//! What the adapter decides, because the Rust CPU does not:
//! - An unused opcode locks: its fetch is logged and `PC` advances as the definition's fetch
//!   does, then the run ends (`final`), as the definition's does (the Rust CPU treats it as a NOP).
//! - `STOP` is not covered (the definition is stuck on it): its fetch is logged and the run ends
//!   (`final`), as the definition's does.
//! - The rest of the console's interrupt requests (the start's `requests`) are set in `IF` between
//!   two steps of the Rust CPU, once their machine cycle has come. A step executes an instruction
//!   and then dispatches, so a request is delivered as the definition delivers it only when it
//!   comes due at that boundary and the CPU is halted or `IME` is clear and not about to be set.
//!   Any other the adapter cannot deliver where the definition does: the answer is
//!   `could-not-apply`, naming the request.
//! - HALT mode idles one machine cycle per halted boundary until a request wakes it. The run
//!   stops before the request's `stop.count`-th halted boundary, where the corpus's runs stop
//!   (corpus/FLOW.toml `[stop]`: before the rule `boundary_halted`, the count-th time).
//! - The interrupt logic's `IE`/`IF` accesses are not bus cycles
//!   ([`memory::Bus::read_interrupt_register`]) and are not logged.
//!
//! The Rust CPU executes each instruction's accesses at once, without the internal cycles, so
//! its log has no idle events and reads may be ordered differently: compare its `bus` under the
//! write view (`foldiboy-sm83-views`).

use std::collections::{BTreeMap, VecDeque};
use std::io::Read;
use std::{env, io};

use cpu::GbCpu;
use memory::Bus;
use serde_json::{Value as Json, json};

/// The opcodes no instruction uses (Pan Docs): executing one locks the CPU.
const UNUSED: [u8; 11] = [0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD];
const STOP: u8 = 0x10;
const IE: u16 = 0xFFFF;
const IF: u16 = 0xFF0F;
/// The five interrupts' bits in `IE` and `IF`, as the definition's `INTERRUPT_BITS`.
const INTERRUPT_BITS: u8 = 0x1F;
/// `IF`'s bits 7 to 5, which read as 1, as the definition's `IF_UNUSED_BITS`.
const IF_UNUSED_BITS: u8 = 0xE0;
const REGISTERS: [&str; 7] = ["A", "B", "C", "D", "E", "H", "L"];
/// The protocol `krs kink compare` speaks.
const PROTOCOL: &str = "kink-implementation/1";
/// The instructions a run may execute before the adapter gives up: no corpus program comes near.
const BUDGET: u64 = 1_000_000;

/// The CPU stage's memory, recording each bus access as an event.
struct CorpusBus {
    rom: Vec<u8>,
    memory: BTreeMap<u16, u8>,
    log: Vec<Json>,
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
        self.log.push(json!({ "Read": { "addr": addr, "value": value } }));
        value
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.log.push(json!({ "Write": { "addr": addr, "value": value } }));
        if usize::from(addr) >= self.rom.len() {
            self.memory.insert(addr, if addr == IF { value | IF_UNUSED_BITS } else { value });
        }
    }

    fn read_interrupt_register(&mut self, addr: u16) -> u8 {
        self.load(addr)
    }

    fn write_interrupt_register(&mut self, addr: u16, value: u8) {
        self.memory.insert(addr, if addr == IF { value | IF_UNUSED_BITS } else { value });
    }
}

/// Whether an enabled interrupt is requested: `IE` and `IF` share a set bit.
fn interrupt_requested(bus: &CorpusBus) -> bool {
    let byte = |addr| bus.memory.get(&addr).copied().unwrap_or(0);
    byte(IE) & byte(IF) & INTERRUPT_BITS != 0
}

fn int<T: TryFrom<u64>>(value: &Json, what: &str) -> Result<T, String> {
    value.as_u64().and_then(|value| T::try_from(value).ok()).ok_or(format!("{what}: {value} is out of range"))
}

fn boolean(value: &Json, what: &str) -> Result<bool, String> {
    value.as_bool().ok_or(format!("{what}: {value} is not a Bool"))
}

/// A `Map<Int, Int>` of bytes: its canonical document is an object keyed by the decimal key, or the
/// pairs form (`$map`).
fn byte_map(document: &Json) -> Result<BTreeMap<u16, u8>, String> {
    if let Some(pairs) = document.get("$map").and_then(Json::as_array) {
        return pairs.iter().map(|pair| Ok((int(&pair[0], "address")?, int(&pair[1], "memory byte")?))).collect();
    }
    let object = document.as_object().ok_or(format!("memory: not a map document: {document}"))?;
    object
        .iter()
        .map(|(key, value)| Ok((key.parse().map_err(|_| format!("memory key {key}"))?, int(value, "memory byte")?)))
        .collect()
}

fn hex(text: &str) -> Result<Vec<u8>, String> {
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&text[at..at + 2], 16).map_err(|error| error.to_string()))
        .collect()
}

/// The requests of a start's `requests` cell, each `{"At": {"cycle": .., "bits": ..}}`, in order.
fn requests(document: &Json) -> Result<VecDeque<(u64, u8)>, String> {
    let items = document.as_array().ok_or(format!("requests: not a list document: {document}"))?;
    items.iter().map(|item| Ok((int(&item["At"]["cycle"], "request cycle")?, int(&item["At"]["bits"], "request bits")?))).collect()
}

/// How a run ended.
enum End {
    /// Before the `stop.count`-th halted boundary.
    Stopped,
    /// On its own: locked, at `STOP`, or stuck on a byte memory does not hold.
    Final,
    /// A request came due where the Rust CPU cannot deliver it as the definition does.
    CouldNotApply(String),
}

/// Runs the CPU from the definition's start (its cells) and returns how the run ended and its
/// end's cells, or why it has no end.
fn run(start: &Json, stop: Option<u64>) -> Result<(End, Json), String> {
    let rom = hex(start["rom"]["$hex"].as_str().ok_or("rom: not a $hex document")?)?;
    let memory = byte_map(&start["memory"])?;
    let mut cpu = GbCpu::new(CorpusBus { rom, memory, log: Vec::new(), undefined: None });
    let regs = &start["regs"];
    let [a, b, c, d, e, h, l] = REGISTERS.map(|name| int::<u8>(&regs[name], name));
    let r = &mut cpu.regs;
    (r.a, r.b, r.c, r.d, r.e, r.h, r.l) = (a?, b?, c?, d?, e?, h?, l?);
    let flags = &start["f"]["Flags"];
    let bit = |name: &str, weight: u8| boolean(&flags[name], name).map(|set| if set { weight } else { 0 });
    r.f = bit("z", 0x80)? | bit("n", 0x40)? | bit("h", 0x20)? | bit("c", 0x10)?;
    r.pc = int(&start["pc"], "pc")?;
    r.sp = int(&start["sp"], "sp")?;
    cpu.ime = boolean(&start["ime"], "ime")?;
    // After EI the definition's boundary holds ei_pending; the Rust CPU's counter is at 0 there.
    cpu.ime_delay = boolean(&start["ei_pending"], "ei_pending")?.then_some(0);
    cpu.halted = boolean(&start["halted"], "halted")?;
    cpu.halt_bug = boolean(&start["halt_bug"], "halt_bug")?;

    let mut pending = requests(&start["requests"])?;
    let (mut instructions, mut cycles, mut halted_boundaries) = (0, 0, 0);
    let end = loop {
        while let Some(&(cycle, bits)) = pending.front().filter(|(cycle, _)| *cycle <= cycles) {
            let exact = cycle == cycles && (cpu.halted || (!cpu.ime && cpu.ime_delay.is_none()));
            if !exact {
                let reason = format!(
                    "the request of {bits:#04x} at machine cycle {cycle} comes due inside an instruction or a dispatch, \
                     or where IME is set or about to be: this CPU delivers a request only between two steps"
                );
                return Ok((End::CouldNotApply(reason), Json::Null));
            }
            *cpu.bus.memory.entry(IF).or_insert(0) |= bits;
            pending.pop_front();
        }
        if cpu.halted && !interrupt_requested(&cpu.bus) {
            // A halted boundary: the definition idles one machine cycle.
            halted_boundaries += 1;
            if stop == Some(halted_boundaries) {
                break End::Stopped;
            }
            if pending.is_empty() && stop.is_none() {
                return Err("HALT mode with no request left never ends, and the corpus has no [stop]".into());
            }
            cycles += 1;
            continue;
        }
        if cpu.bus.undefined.is_some() {
            break End::Final;
        }
        if !cpu.halted {
            let pc = cpu.regs.pc;
            if cpu.bus.byte(pc).is_some_and(|op| UNUSED.contains(&op) || op == STOP) {
                cpu.bus.read(pc);
                if !cpu.halt_bug {
                    cpu.regs.pc = pc.wrapping_add(1);
                }
                cpu.halt_bug = false;
                break End::Final;
            }
        }
        if instructions == BUDGET {
            return Err(format!("no end after {BUDGET} instructions"));
        }
        cycles += u64::from(cpu.step());
        instructions += 1;
    };

    let mut cells = start.clone();
    let r = &cpu.regs;
    cells["regs"] = json!({ "A": r.a, "B": r.b, "C": r.c, "D": r.d, "E": r.e, "H": r.h, "L": r.l });
    cells["f"] = json!({ "Flags": { "z": r.zf(), "n": r.nf(), "h": r.hf(), "c": r.cf() } });
    cells["pc"] = json!(r.pc);
    cells["sp"] = json!(r.sp);
    cells["ime"] = json!(cpu.ime);
    cells["ei_pending"] = json!(cpu.ime_delay.is_some());
    cells["halted"] = json!(cpu.halted);
    cells["halt_bug"] = json!(cpu.halt_bug);
    cells["memory"] = json!({ "$map": cpu.bus.memory.iter().map(|(addr, value)| json!([addr, value])).collect::<Vec<_>>() });
    cells["bus"] = Json::Array(std::mem::take(&mut cpu.bus.log));
    cells["requests"] = pending.iter().map(|(cycle, bits)| json!({ "At": { "cycle": cycle, "bits": bits } })).collect();
    if let Some(cells) = cells.as_object_mut() {
        // The definition's internals (k) and the Rust CPU has no W and Z: FLOW.toml's `omits`.
        for omitted in ["k", "w", "z"] {
            cells.remove(omitted);
        }
    }
    Ok((end, cells))
}

/// Answers one request.
fn answer(request: &Json) -> Result<Json, String> {
    if request["protocol"] != PROTOCOL {
        return Err(format!("protocol {}, not {PROTOCOL}", request["protocol"]));
    }
    let start = request.get("start").ok_or("no start: FLOW.toml must give this implementation the start")?;
    let stop = request.get("stop").map(|stop| int(&stop["count"], "stop count")).transpose()?;
    Ok(match run(start, stop)? {
        (End::Stopped, cells) => json!({ "end": "stopped", "cells": cells }),
        (End::Final, cells) => json!({ "end": "final", "cells": cells }),
        (End::CouldNotApply(reason), _) => json!({ "end": "could-not-apply", "reason": reason }),
    })
}

fn main() {
    if env::args().skip(1).collect::<Vec<_>>() != ["--kink"] {
        eprintln!("usage: sm83-rust-cpu --kink < REQUEST.json (the protocol {PROTOCOL})");
        std::process::exit(2);
    }
    let mut text = String::new();
    let result = io::stdin()
        .read_to_string(&mut text)
        .map_err(|error| format!("stdin: {error}"))
        .and_then(|_| serde_json::from_str(&text).map_err(|error| format!("request: {error}")))
        .and_then(|request: Json| answer(&request));
    match result {
        Ok(answer) => println!("{answer}"),
        Err(reason) => {
            eprintln!("sm83-rust-cpu: {reason}");
            std::process::exit(1);
        }
    }
}
