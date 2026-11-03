//! Runs the SM83 KRS semantics against the SingleStepTests sm83 corpus.
//!
//! Each vector is one instruction from an initial state.
//! The harness configures the definition with the vector's registers and
//! memory at an instruction boundary, runs it to the next boundary, and
//! compares registers, flags,
//! `PC`, `SP`, `IME`, the pending `EI`, the listed memory, and the bus log.
//! An idle cycle is compared by kind only: the corpus records the
//! address and data lines left on the bus, which the model does not track.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, fs, thread};

use krs_exec::{ConfigInputs, ConfigValue, Semantics, StateView, View};
use krs_ir::measure::Timings;
use serde_json::{Value, json};
use sm83_vectors::{Access, Names, REGISTERS, bool as cell_bool, int as cell_int, int_map, load, next_boundary};

struct Options {
    definition_ir: PathBuf,
    corpus: PathBuf,
    files: Vec<String>,
    limit: usize,
    jobs: usize,
    report: Option<PathBuf>,
}

fn usage() -> ! {
    eprintln!(
        "usage: sm83-vectors --definition-ir PATH --corpus DIR [--files a,b,..] [--limit N] [--jobs N] [--report PATH]"
    );
    std::process::exit(2)
}

fn options() -> Options {
    let mut args = env::args().skip(1);
    let mut options = Options {
        definition_ir: PathBuf::new(),
        corpus: PathBuf::new(),
        files: Vec::new(),
        limit: usize::MAX,
        jobs: thread::available_parallelism().map_or(1, usize::from),
        report: None,
    };
    while let Some(flag) = args.next() {
        let value = args.next().unwrap_or_else(|| usage());
        match flag.as_str() {
            "--definition-ir" => options.definition_ir = value.into(),
            "--corpus" => options.corpus = value.into(),
            "--files" => options.files = value.split(',').map(str::to_owned).collect(),
            "--limit" => options.limit = value.parse().unwrap_or_else(|_| usage()),
            "--jobs" => options.jobs = value.parse().unwrap_or_else(|_| usage()),
            "--report" => options.report = Some(value.into()),
            _ => usage(),
        }
    }
    if options.definition_ir.as_os_str().is_empty() || options.corpus.as_os_str().is_empty() {
        usage();
    }
    if options.files.is_empty() {
        let mut files = fs::read_dir(&options.corpus)
            .expect("corpus directory")
            .filter_map(|entry| {
                let name = entry.ok()?.file_name().into_string().ok()?;
                name.strip_suffix(".json").map(str::to_owned)
            })
            .collect::<Vec<_>>();
        files.sort();
        options.files = files;
    }
    options
}

/// The observable state the corpus records, as decoded from either side.
#[derive(Debug, PartialEq)]
struct Observed {
    registers: BTreeMap<String, i64>,
    ime: bool,
    /// Whether `EI` has scheduled `IME` to be set after the next instruction.
    ei_pending: bool,
    memory: BTreeMap<i64, i64>,
    bus: Vec<String>,
}

fn int(value: &Value) -> i64 {
    value.as_i64().expect("integer field")
}

/// `IE` and `IF`, which every boundary reads.
const INTERRUPT_REGISTERS: [i64; 2] = [0xFFFF, 0xFF0F];

fn inputs(names: &Names, initial: &Value) -> ConfigInputs {
    let registers = names.register_file(REGISTERS.map(|name| int(&initial[name])));
    let mut ram: BTreeMap<i64, i64> = initial["ram"]
        .as_array()
        .expect("ram list")
        .iter()
        .map(|entry| (int(&entry[0]), int(&entry[1])))
        .collect();
    // The corpus does not model interrupts. A register a vector does not
    // list reads as zero, so no interrupt is requested at the boundary: no
    // vector lists both.
    for address in INTERRUPT_REGISTERS {
        ram.entry(address).or_insert(0);
    }
    let memory = krs_exec::Value::map(ram.into_iter().map(|(address, value)| (address.into(), value.into())));
    let mut variables: Vec<(String, ConfigValue)> = vec![("REGS".to_owned(), registers.into())];
    variables.extend(
        ["f", "pc", "sp"]
            .iter()
            .map(|name| (name.to_uppercase(), krs_exec::Value::from(int(&initial[*name])).into())),
    );
    variables.push(("IME".to_owned(), krs_exec::Value::from(int(&initial["ime"]) != 0).into()));
    for name in ["EI_PENDING", "HALTED", "HALT_BUG"] {
        variables.push((name.to_owned(), krs_exec::Value::from(false).into()));
    }
    variables.push(("MEMORY".to_owned(), memory.into()));
    ConfigInputs {
        variables,
        ..ConfigInputs::default()
    }
}

fn expected(vector: &Value) -> Observed {
    let last = &vector["final"];
    let registers = REGISTERS
        .iter()
        .chain(&["f", "pc", "sp"])
        .map(|name| ((*name).to_owned(), int(&last[*name])))
        .collect();
    let memory = last["ram"]
        .as_array()
        .expect("ram list")
        .iter()
        .map(|entry| (int(&entry[0]), int(&entry[1])))
        .collect();
    let bus = vector["cycles"]
        .as_array()
        .expect("cycle list")
        .iter()
        .map(|cycle| match cycle[2].as_str().expect("cycle kind") {
            "r-m" => format!("read({}, {})", int(&cycle[0]), int(&cycle[1])),
            "-wm" => format!("write({}, {})", int(&cycle[0]), int(&cycle[1])),
            "---" => "idle".to_owned(),
            other => format!("unknown({other})"),
        })
        .collect();
    Observed {
        registers,
        ime: int(&last["ime"]) != 0,
        ei_pending: last.get("ei").is_some_and(|ei| int(ei) != 0),
        memory,
        bus,
    }
}

fn observed(names: &Names, view: &StateView) -> Result<Observed, String> {
    let mut registers = names.registers(view)?;
    for name in ["f", "pc", "sp"] {
        registers.insert(name.to_owned(), cell_int(view, name)?);
    }
    let bus = names
        .bus(view)?
        .into_iter()
        .map(|access| match access {
            Access::Read { addr, value } => format!("read({addr}, {value})"),
            // The corpus records the written byte, not the byte it replaces.
            Access::Write { addr, value } => format!("write({addr}, {value})"),
            Access::Idle => "idle".to_owned(),
        })
        .collect();
    Ok(Observed {
        registers,
        ime: cell_bool(view, "ime")?,
        ei_pending: cell_bool(view, "ei_pending")?,
        memory: int_map(view, "memory")?,
        bus,
    })
}

/// Restricts the model's memory to the addresses the vector lists.
fn listed(mut actual: Observed, expected: &Observed) -> Observed {
    actual.memory.retain(|address, _| expected.memory.contains_key(address));
    actual
}

fn run_vector(semantics: &mut Semantics, names: &Names, vector: &Value) -> Result<(), String> {
    let state = semantics
        .configure(&inputs(names, &vector["initial"]))
        .map_err(|error| format!("configure: {error}"))?;
    let after = next_boundary(semantics, &state, 64)?.ok_or("no boundary rule within 64 steps")?;
    let View::State(view) = semantics.view(&after).map_err(|error| error.to_string())? else {
        return Err("the result is #Bottom".to_owned());
    };
    let expected = expected(vector);
    let actual = listed(observed(names, &view)?, &expected);
    if actual == expected {
        Ok(())
    } else {
        Err(format!("expected {expected:?}\n  actual {actual:?}"))
    }
}

fn run_file(semantics: &mut Semantics, names: &Names, options: &Options, file: &str) -> Value {
    let path = options.corpus.join(format!("{file}.json"));
    let vectors: Vec<Value> =
        serde_json::from_slice(&fs::read(&path).expect("vector file")).expect("vector JSON");
    let mut passed = 0;
    let mut failures = Vec::new();
    let total = vectors.len().min(options.limit);
    for vector in vectors.iter().take(total) {
        match run_vector(semantics, names, vector) {
            Ok(()) => passed += 1,
            Err(reason) if failures.len() < 3 => {
                failures.push(json!({ "name": vector["name"], "reason": reason }));
            }
            Err(_) => {}
        }
    }
    json!({ "file": file, "total": total, "passed": passed, "failures": failures })
}

fn main() {
    let options = options();
    let (definition, stage) = load(&options.definition_ir);
    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::new());
    thread::scope(|scope| {
        for _ in 0..options.jobs.min(options.files.len()) {
            thread::Builder::new()
                .stack_size(64 << 20)
                .spawn_scoped(scope, || {
                    let mut semantics =
                        Semantics::from_stage(&definition, stage.clone(), None, &mut Timings::disabled())
                            .expect("load stage");
                    let names = Names::new(semantics.values());
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(file) = options.files.get(index) else { break };
                        let result = run_file(&mut semantics, &names, &options, file);
                        eprintln!("{} {}/{}", file, result["passed"], result["total"]);
                        results.lock().expect("results").push(result);
                    }
                })
                .expect("worker thread");
        }
    });
    let mut results = results.into_inner().expect("results");
    results.sort_by(|a, b| a["file"].as_str().cmp(&b["file"].as_str()));
    let total: u64 = results.iter().map(|r| r["total"].as_u64().unwrap_or(0)).sum();
    let passed: u64 = results.iter().map(|r| r["passed"].as_u64().unwrap_or(0)).sum();
    let report = json!({ "total": total, "passed": passed, "files": results });
    let text = serde_json::to_string_pretty(&report).expect("report");
    match &options.report {
        Some(path) => fs::write(path, text + "\n").expect("write report"),
        None => println!("{text}"),
    }
    println!("passed {passed}/{total}");
}
