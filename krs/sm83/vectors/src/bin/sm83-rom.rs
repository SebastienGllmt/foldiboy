//! Runs a Game Boy test ROM on the SM83 KRS semantics for at most a number
//! of rewrite steps and reports the final registers.
//!
//! The machine runs forever; a mooneye test ends by looping on itself, so a
//! run normally ends at the depth bound.
//!
//! The address space is flat: the ROM fills `$0000-$7FFF` and every other
//! byte starts at zero, except the I/O bytes listed in `IO_INIT`. The CPU
//! starts in the state the DMG boot ROM leaves.
//!
//! A mooneye test signals its result in `B C D E H L` before it loops on
//! itself: `3 5 8 13 21 34` is a pass, all `$42` a failure.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::{env, fs, thread};

use krs_exec::{ConfigInputs, ConfigValue, Halt, RunOptions, Semantics, Value, View};
use krs_ir::measure::Timings;
use serde_json::json;
use sm83_vectors::{Access, Names, bool as cell_bool, cell, int as cell_int, int_map, load};

/// I/O bytes whose initial value matters to the tests.
///
/// `LY` reads `$FF` so the mooneye harness treats the PPU as absent and
/// skips its display code; `IF` is the post-boot value.
const IO_INIT: [(usize, u8); 2] = [(0xFF44, 0xFF), (0xFF0F, 0xE1)];

const PASS: [i64; 6] = [3, 5, 8, 13, 21, 34];
const FAIL: [i64; 6] = [0x42; 6];

fn usage() -> ! {
    eprintln!("usage: sm83-rom --definition-ir PATH --depth N [--report PATH] ROM...");
    std::process::exit(2)
}

fn inputs(names: &Names, rom: &[u8]) -> ConfigInputs {
    let mut memory = vec![0u8; 0x10000];
    memory[..rom.len().min(0x8000)].copy_from_slice(&rom[..rom.len().min(0x8000)]);
    for (address, value) in IO_INIT {
        memory[address] = value;
    }
    let memory = Value::map(
        memory
            .iter()
            .enumerate()
            .map(|(address, value)| (Value::from(address as i64), Value::from(i64::from(*value)))),
    );
    let variables: [(&str, ConfigValue); 9] = [
        ("REGS", names.register_file([1, 0, 19, 0, 216, 1, 77]).into()),
        ("F", Value::from(176).into()),
        ("PC", Value::from(256).into()),
        ("SP", Value::from(65534).into()),
        ("IME", Value::from(false).into()),
        ("EI_PENDING", Value::from(false).into()),
        ("HALTED", Value::from(false).into()),
        ("HALT_BUG", Value::from(false).into()),
        ("MEMORY", memory.into()),
    ];
    ConfigInputs {
        variables: variables.into_iter().map(|(name, value)| (name.to_owned(), value)).collect(),
        ..ConfigInputs::default()
    }
}

/// The registers a mooneye test signals its result in, read when the run ends.
struct Outcome {
    verdict: &'static str,
    pc: i64,
    bcdehl: [i64; 6],
}

fn run(semantics: &mut Semantics, names: &Names, rom: &[u8], depth: u64) -> Result<Outcome, String> {
    let state = semantics.configure(&inputs(names, rom)).map_err(|error| format!("configure: {error}"))?;
    let report = semantics
        .run(&state, RunOptions { depth: Some(depth), ..RunOptions::default() })
        .map_err(|error| format!("run: {error}"))?;
    let View::State(view) = semantics.view(&report.state).map_err(|error| error.to_string())? else {
        return Err("the result is #Bottom".to_owned());
    };
    if report.halt != Halt::DepthBound {
        let k = format!("{:?}", cell(&view, "k")?);
        return Err(format!("{:?} after {} steps; <k> is {}", report.halt, report.steps, &k[..k.len().min(1500)]));
    }
    let registers = names.registers(&view)?;
    let result = ["b", "c", "d", "e", "h", "l"].map(|name| registers[name]);
    let pc = cell_int(&view, "pc")?;
    let verdict = if result == PASS {
        "pass"
    } else if result == FAIL {
        "fail"
    } else {
        "unfinished"
    };
    if env::var_os("SM83_BUS").is_some() {
        for access in names.bus(&view)? {
            match access {
                Access::Read { addr, value } => eprintln!("read {addr:04x} {value:02x}"),
                Access::Write { addr, value } => eprintln!("write {addr:04x} {value:02x}"),
                Access::Idle => eprintln!("idle"),
            }
        }
    }
    if let Some(range) = env::var("SM83_DUMP").ok() {
        let (start, len) = range.split_once(':').expect("SM83_DUMP=start:len, hexadecimal");
        let start = i64::from_str_radix(start, 16).expect("start");
        let len = i64::from_str_radix(len, 16).expect("len");
        let memory = int_map(&view, "memory")?;
        let bytes = (start..start + len)
            .map(|address| memory.get(&address).map_or("??".to_owned(), |byte| format!("{byte:02x}")))
            .collect::<Vec<_>>();
        eprintln!("{start:04x}: {}", bytes.join(" "));
        eprintln!("halted={}", cell_bool(&view, "halted")?);
    }
    Ok(Outcome { verdict, pc, bcdehl: result })
}

fn main() {
    let mut args = env::args().skip(1);
    let mut definition_ir = PathBuf::new();
    let mut depth = 0;
    let mut report = None;
    let mut roms = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--definition-ir" => definition_ir = args.next().unwrap_or_else(|| usage()).into(),
            "--depth" => depth = args.next().and_then(|n| n.parse().ok()).unwrap_or_else(|| usage()),
            "--report" => report = Some(PathBuf::from(args.next().unwrap_or_else(|| usage()))),
            _ => roms.push(PathBuf::from(arg)),
        }
    }
    if definition_ir.as_os_str().is_empty() || roms.is_empty() {
        usage();
    }
    let (definition, stage) = load(&definition_ir);
    let results = Mutex::new(Vec::new());
    thread::scope(|scope| {
        for path in &roms {
            let (definition, stage, results) = (&definition, stage.clone(), &results);
            thread::Builder::new()
                .stack_size(256 << 20)
                .spawn_scoped(scope, move || {
                    let mut semantics = Semantics::from_stage(definition, stage, None, &mut Timings::disabled())
                        .expect("load stage");
                    let names = Names::new(semantics.values());
                    let rom = fs::read(path).expect("ROM");
                    let name = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("?");
                    let record = match run(&mut semantics, &names, &rom, depth) {
                        Ok(Outcome { verdict, pc, bcdehl }) => {
                            println!("{name}: {verdict} pc={pc:#06x} bcdehl={bcdehl:?}");
                            json!({ "rom": rom_id(path), "verdict": verdict, "pc": pc, "bcdehl": bcdehl })
                        }
                        Err(reason) => {
                            println!("{name}: error {reason}");
                            json!({ "rom": rom_id(path), "verdict": "error", "reason": reason })
                        }
                    };
                    results.lock().expect("results").push(record);
                })
                .expect("worker thread");
        }
    });
    if let Some(path) = report {
        let mut results = results.into_inner().expect("results");
        results.sort_by(|a, b| a["rom"].as_str().cmp(&b["rom"].as_str()));
        let passed = results.iter().filter(|r| r["verdict"] == "pass").count();
        let report = json!({ "depth": depth, "total": results.len(), "passed": passed, "roms": results });
        fs::write(path, serde_json::to_string_pretty(&report).expect("report") + "\n").expect("write report");
    }
}

/// The ROM's path below the suite's `acceptance/` directory, without extension, as the suite names its tests.
fn rom_id(path: &Path) -> String {
    let text = path.with_extension("").to_string_lossy().into_owned();
    match text.rfind("acceptance/") {
        Some(start) => text[start + "acceptance/".len()..].to_owned(),
        None => text,
    }
}
