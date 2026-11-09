//! Runs the Rust emulator's CPU (`lib/crates/cpu`) on corpus cases and writes each run's end as a
//! krs end document, which `krs compare` compares with the SM83 definition's run of the case.
//!
//!     sm83-rust-cpu --out DIR [--budget N] START.json...
//!
//! writes `DIR/<id>.json` for each `<id>.json`, and prints the end class and the cost of each run.
//!
//! A START is the case's initial state as the definition builds it, the end document of
//! `krs run CARTRIDGE --depth 0 --output end` (the definition's initializer calls the
//! specification's boot functions, which krs evaluates). The end document written is that document with the cells the
//! CPU changes written over it, so every cell is in the encoding `krs compare` reads.
//!
//! The CPU runs over the CPU stage's memory model, the one the definition states, so a
//! comparison is between the two CPUs and not between two memory maps: the ROM region is
//! read-only, every other byte holds only what the initial state or a write put there, and a
//! read of any other byte has no value, which ends the run stuck, as it does in the definition.
//! `IF`'s bits 7 to 5 read as 1: a write to it stores them set, as the definition's `Latch` does.
//!
//! What the adapter decides, because the Rust CPU does not:
//! - An unused opcode locks: its fetch is logged and `PC` advances as the definition's fetch
//!   does, then the run ends stuck with `k` holding the lock, as the definition's does (the Rust
//!   CPU treats it as a NOP).
//! - `STOP` is not covered (the definition is stuck on it): its fetch is logged and the run ends
//!   stuck with `k` holding `STOP`, as the definition's does.
//! - The rest of the console's interrupt requests (the start's `requests`) are set in `IF` between
//!   two steps of the Rust CPU, once their machine cycle has come. A step executes an instruction
//!   and then dispatches, so a request is delivered as the definition delivers it only when it
//!   comes due at that boundary and the CPU is halted or `IME` is clear and not about to be set;
//!   any other is late, and counted in `DIR/late.json` (scripts/compare.ts reports such a case
//!   instead of requiring it equal).
//! - HALT mode idles until the next request comes due; with none left, it never ends at the CPU
//!   stage: the run stops there, as the corpus's runs stop at their second halted boundary
//!   (corpus/FLOW.toml `[stop]`).
//! - The interrupt logic's `IE`/`IF` accesses are not bus cycles
//!   ([`memory::Bus::read_interrupt_register`]) and are not logged.
//!
//! The Rust CPU executes each instruction's accesses at once, without the internal cycles, so
//! its log has no idle events and reads may be ordered differently: compare its `bus` under the
//! write view (`foldiboy-sm83-views`).

use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::{env, fs};

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

fn usage() -> ! {
    eprintln!("usage: sm83-rust-cpu --out DIR [--budget INSTRUCTIONS] START.json...");
    std::process::exit(2)
}

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

/// How the run ended, and what it cost.
struct Outcome {
    end: &'static str,
    instructions: u64,
    cycles: u64,
    /// The requests delivered later than the definition delivers them.
    late: u64,
}

/// Runs the CPU from the initial end document and returns the end document of its run.
fn run(initial: &Json, budget: u64) -> Result<(Json, Outcome), String> {
    let cells = &initial["cells"];
    let rom = hex(cells["rom"]["$hex"].as_str().ok_or("rom: not a $hex document")?)?;
    let memory = byte_map(&cells["memory"])?;
    let mut cpu = GbCpu::new(CorpusBus { rom, memory, log: Vec::new(), undefined: None });
    let regs = &cells["regs"];
    let [a, b, c, d, e, h, l] = REGISTERS.map(|name| int::<u8>(&regs[name], name));
    let r = &mut cpu.regs;
    (r.a, r.b, r.c, r.d, r.e, r.h, r.l) = (a?, b?, c?, d?, e?, h?, l?);
    let flags = &cells["f"]["Flags"];
    let bit = |name: &str, weight: u8| boolean(&flags[name], name).map(|set| if set { weight } else { 0 });
    r.f = bit("z", 0x80)? | bit("n", 0x40)? | bit("h", 0x20)? | bit("c", 0x10)?;
    r.pc = int(&cells["pc"], "pc")?;
    r.sp = int(&cells["sp"], "sp")?;
    cpu.ime = boolean(&cells["ime"], "ime")?;
    // After EI the definition's boundary holds ei_pending; the Rust CPU's counter is at 0 there.
    cpu.ime_delay = boolean(&cells["ei_pending"], "ei_pending")?.then_some(0);
    cpu.halted = boolean(&cells["halted"], "halted")?;
    cpu.halt_bug = boolean(&cells["halt_bug"], "halt_bug")?;

    let mut pending = requests(&cells["requests"])?;

    let mut outcome = Outcome { end: "bound", instructions: 0, cycles: 0, late: 0 };
    let mut k = None;
    loop {
        while let Some(&(cycle, bits)) = pending.front().filter(|(cycle, _)| *cycle <= outcome.cycles) {
            let exact = cycle == outcome.cycles && (cpu.halted || (!cpu.ime && cpu.ime_delay.is_none()));
            outcome.late += u64::from(!exact);
            *cpu.bus.memory.entry(IF).or_insert(0) |= bits;
            pending.pop_front();
        }
        if cpu.halted && !interrupt_requested(&cpu.bus) {
            if let Some(&(cycle, _)) = pending.front() {
                // HALT mode idles one machine cycle at a time until the request comes due.
                outcome.cycles = cycle;
                continue;
            }
            outcome.end = "stopped";
            k = Some(json!({ "$seq": [{ "Boundary": [] }] }));
            break;
        }
        if cpu.bus.undefined.is_some() {
            outcome.end = "stuck";
            break;
        }
        if !cpu.halted {
            let pc = cpu.regs.pc;
            if let Some(opcode) = cpu.bus.byte(pc).filter(|op| UNUSED.contains(op) || *op == STOP) {
                cpu.bus.read(pc);
                if !cpu.halt_bug {
                    cpu.regs.pc = pc.wrapping_add(1);
                }
                cpu.halt_bug = false;
                if opcode == STOP {
                    outcome.end = "stuck";
                    k = Some(json!({ "$seq": [{ "Stop": [] }, { "Boundary": [] }] }));
                } else {
                    outcome.end = "locked";
                    k = Some(json!({ "$seq": [{ "Locked": [] }, { "Boundary": [] }] }));
                }
                break;
            }
        }
        if outcome.instructions == budget {
            break;
        }
        outcome.cycles += u64::from(cpu.step());
        outcome.instructions += 1;
    }

    let mut end = initial.clone();
    end["halt"] = json!(match outcome.end {
        "bound" => "depth-bound",
        "stopped" => "stopped",
        _ => "stuck",
    });
    if outcome.end == "stopped" {
        end["rule"] = json!("crate::interrupt::boundary_halted");
    }
    let cells = &mut end["cells"];
    // Only the computation at a lock, at STOP and in HALT mode is known; any other end leaves the CPU somewhere the definition
    // names by its own micro-operations, so `k` is stated as empty and differs.
    cells["k"] = k.unwrap_or_else(|| json!({ "$seq": [] }));
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
        // Unobservable: the Rust CPU has no W and Z, and no comparison reads them.
        cells.remove("w");
        cells.remove("z");
    }
    Ok((end, outcome))
}

fn main() {
    let mut args = env::args().skip(1);
    let (mut out, mut budget, mut starts) = (None, 1_000_000, Vec::new());
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = Some(PathBuf::from(args.next().unwrap_or_else(|| usage()))),
            "--budget" => budget = args.next().and_then(|n| n.parse().ok()).unwrap_or_else(|| usage()),
            _ if arg.starts_with("--") => usage(),
            _ => starts.push(PathBuf::from(arg)),
        }
    }
    let Some(out) = out else { usage() };
    fs::create_dir_all(&out).expect("output directory");
    let mut failed = false;
    let mut late = serde_json::Map::new();
    for path in &starts {
        let id = path.file_stem().map_or_else(|| path.display().to_string(), |id| id.to_string_lossy().into_owned());
        let result = fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))
            .and_then(|text| serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display())))
            .and_then(|initial| run(&initial, budget));
        match result {
            Ok((end, outcome)) => {
                let text = serde_json::to_string_pretty(&end).expect("end document") + "\n";
                fs::write(out.join(format!("{id}.json")), text).expect("write end document");
                let note = if outcome.late > 0 { format!(", {} request(s) late", outcome.late) } else { String::new() };
                println!("{id}: {} after {} instructions, {} M-cycles{note}", outcome.end, outcome.instructions, outcome.cycles);
                if outcome.late > 0 {
                    late.insert(id, json!(outcome.late));
                }
            }
            Err(reason) => {
                eprintln!("{id}: {reason}");
                failed = true;
            }
        }
    }
    fs::write(out.join("late.json"), Json::Object(late).to_string() + "\n").expect("write late.json");
    std::process::exit(i32::from(failed));
}
