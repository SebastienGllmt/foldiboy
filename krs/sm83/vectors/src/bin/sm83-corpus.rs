//! Runs the corpus (`corpus/` at the repository root) on every system its manifest names and
//! compares the systems pair by pair (draft-personal/corpus-contract sm83-refactor R7).
//!
//! A case is a cartridge ROM; every system starts from the machine state the manifest's
//! machine document gives, a krs case document of the SM83 definition's inputs to which the
//! runner adds the cartridge. Each system's run becomes an observation (end class, event log,
//! end state); two observations are compared by the comparator definition (`sm83-compare`)
//! under the pair's tier and normalizations.
//!
//! Systems:
//! - `krs`: the SM83 definition through krs-exec, up to `budget` rewrite steps.
//! - `rust-cpu`: the Rust emulator's CPU over the definition's memory model, up to `budget`
//!   instructions (`sm83_vectors::rust_cpu`).
//! - `file`: an observation another system recorded, `<dir>/<case>.json` relative to the
//!   manifest, in the comparator's input encoding.
//!
//! Costs (steps, instructions, cycles, wall time) are reported, never compared.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;
use std::{env, fs, thread};

use krs_exec::inputs::{Case, InputFiles, NoFiles};
use krs_exec::{Halt, RunOptions, Semantics, Value, View};
use krs_ir::measure::Timings;
use serde::Deserialize;
use serde_json::{Value as Json, json};
use sm83_vectors::observation::{EndClass, EndState, Observation, k_head};
use sm83_vectors::rust_cpu::{self, InitialState};
use sm83_vectors::{Names, cell, load};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    /// The machine document: a krs case of the SM83 definition's inputs, without `rom`.
    machine: PathBuf,
    /// Where the corpus build writes `<case>.gb`.
    roms: PathBuf,
    /// The cases, by id; every directory under `cases/` when absent.
    cases: Option<Vec<String>>,
    system: Vec<System>,
    pair: Vec<Pair>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct System {
    id: String,
    kind: String,
    budget: Option<u64>,
    dir: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pair {
    left: String,
    right: String,
    tier: String,
    #[serde(default)]
    normalizations: Vec<String>,
}

fn usage() -> ! {
    eprintln!(
        "usage: sm83-corpus --corpus MANIFEST --definition-ir PATH --compare-ir PATH \
         [--observations DIR] [--report PATH] [CASE...]"
    );
    std::process::exit(2)
}

/// Reads the files a case document names, relative to the document that names them.
struct Files {
    base: PathBuf,
}

impl InputFiles for Files {
    fn read(&mut self, from: Option<&str>, path: &str) -> Result<(String, Vec<u8>), String> {
        let dir = from.map(|from| Path::new(from).parent().unwrap_or(Path::new("")).to_owned()).unwrap_or(self.base.clone());
        let resolved = dir.join(path);
        let bytes = fs::read(&resolved).map_err(|error| format!("{}: {error}", resolved.display()))?;
        Ok((resolved.to_string_lossy().into_owned(), bytes))
    }
}

/// One system's run of one case.
struct Run {
    observation: Observation,
    cost: Json,
}

fn run_krs(semantics: &mut Semantics, names: &Names, state: &krs_exec::State, budget: u64) -> Result<Run, String> {
    let started = Instant::now();
    let report = semantics
        .run(state, RunOptions { depth: Some(budget), ..RunOptions::default() })
        .map_err(|error| format!("run: {error}"))?;
    let wall = started.elapsed();
    let View::State(view) = semantics.view(&report.state).map_err(|error| error.to_string())? else {
        return Err(format!("{:?} with no state (#Bottom) after {} steps", report.halt, report.steps));
    };
    let end = match report.halt {
        Halt::DepthBound => EndClass::Bound,
        Halt::Stuck if k_head(&view)?.ends_with("Step::Locked") => EndClass::Locked,
        Halt::Stuck => EndClass::Stuck,
        other => return Err(format!("{other:?} after {} steps; <k> head {}", report.steps, k_head(&view)?)),
    };
    let observation = Observation { end, log: names.bus(&view)?, state: EndState::from_krs(names, &view)? };
    Ok(Run { observation, cost: json!({ "rewrite_steps": report.steps, "wall_ms": wall.as_millis() as u64 }) })
}

/// A comparator verdict, as `krs run` prints it.
fn show(value: &Value) -> String {
    match value {
        Value::Ctor(ctor) => {
            let path = ctor.decl().path();
            let name = path.rsplit("::").next().unwrap_or(path);
            if ctor.fields().is_empty() {
                name.to_owned()
            } else {
                let fields: Vec<String> = ctor.fields().iter().map(|(field, value)| format!("{field}: {}", show(value))).collect();
                format!("{name} {{ {} }}", fields.join(", "))
            }
        }
        Value::Int(int) => int.to_string(),
        Value::Bool(bool) => bool.to_string(),
        Value::List(items) | Value::Seq(items) => format!("[{}]", items.iter().map(show).collect::<Vec<_>>().join(", ")),
        other => format!("{other:?}"),
    }
}

fn compare(compare: &mut Semantics, left: &Observation, right: &Observation, pair: &Pair) -> Result<String, String> {
    let normalizations: Vec<Json> = pair.normalizations.iter().map(|name| json!(name)).collect();
    let document = json!({ "inputs": {
        "left": left.to_json(),
        "right": right.to_json(),
        "tier": pair.tier,
        "normalizations": normalizations,
    }});
    let case = Case::from_json(&document.to_string()).map_err(|error| format!("{error:?}"))?;
    let inputs = compare.case_inputs(&case, None, &mut NoFiles).map_err(|error| error.to_string())?;
    let state = compare.configure(&inputs).map_err(|error| error.to_string())?;
    let report = compare.run(&state, RunOptions::default()).map_err(|error| error.to_string())?;
    let View::State(view) = compare.view(&report.state).map_err(|error| error.to_string())? else {
        return Err("the comparison has no result (#Bottom)".to_owned());
    };
    let k = cell(&view, "k")?;
    let verdict = k.as_seq().and_then(<[_]>::first).ok_or(format!("<k> is {k:?}"))?;
    Ok(show(verdict))
}

struct Context<'a> {
    manifest: &'a Manifest,
    root: &'a Path,
    observations: Option<&'a Path>,
}

fn run_case(context: &Context, semantics: &mut Semantics, comparator: &mut Semantics, case_id: &str) -> Json {
    let names = Names::new(semantics.values());
    let manifest = context.manifest;
    let machine_path = context.root.join(&manifest.machine);
    let rom = context.root.join(&manifest.roms).join(format!("{case_id}.gb"));
    let initial = (|| -> Result<_, String> {
        let text = fs::read_to_string(&machine_path).map_err(|error| format!("{}: {error}", machine_path.display()))?;
        let mut case = Case::from_toml(&text).map_err(|error| format!("{error:?}"))?;
        let rom = fs::canonicalize(&rom).map_err(|error| format!("{}: {error}", rom.display()))?;
        case.inputs.push((
            "rom".to_owned(),
            json!({ "$call": { "RomRegion": { "cartridge": { "$file": rom.to_string_lossy() } } } }),
        ));
        let mut files = Files { base: machine_path.parent().unwrap_or(Path::new(".")).to_owned() };
        let inputs = semantics.case_inputs(&case, None, &mut files).map_err(|error| error.to_string())?;
        let state = semantics.configure(&inputs).map_err(|error| format!("configure: {error}"))?;
        let report =
            semantics.run(&state, RunOptions { depth: Some(0), ..RunOptions::default() }).map_err(|error| error.to_string())?;
        let View::State(view) = semantics.view(&report.state).map_err(|error| error.to_string())? else {
            return Err("the initial state is #Bottom".to_owned());
        };
        Ok((state, InitialState::from_krs(&names, &view)?))
    })();
    let (state, initial) = match initial {
        Ok(initial) => initial,
        Err(reason) => return json!({ "case": case_id, "error": reason }),
    };

    let mut runs: BTreeMap<&str, Result<Run, String>> = BTreeMap::new();
    for system in &manifest.system {
        let run = match system.kind.as_str() {
            "krs" => run_krs(semantics, &names, &state, system.budget.unwrap_or(1_000_000)),
            "rust-cpu" => {
                let started = Instant::now();
                rust_cpu::run(&initial, system.budget.unwrap_or(1_000_000)).map(|(observation, cost)| Run {
                    observation,
                    cost: json!({
                        "instructions": cost.instructions,
                        "m_cycles": cost.cycles,
                        "wall_ms": started.elapsed().as_millis() as u64,
                    }),
                })
            }
            "file" => {
                let dir = context.root.join(system.dir.as_deref().unwrap_or(Path::new(".")));
                let path = dir.join(format!("{case_id}.json"));
                fs::read_to_string(&path)
                    .map_err(|error| format!("{}: {error}", path.display()))
                    .and_then(|text| serde_json::from_str(&text).map_err(|error| error.to_string()))
                    .and_then(|document| Observation::from_json(&document))
                    .map(|observation| Run { observation, cost: json!({}) })
            }
            other => Err(format!("unknown system kind {other}")),
        };
        if let (Some(dir), Ok(run)) = (context.observations, &run) {
            let dir = dir.join(&system.id);
            let written = fs::create_dir_all(&dir).and_then(|()| {
                fs::write(dir.join(format!("{case_id}.json")), serde_json::to_string_pretty(&run.observation.to_json()).unwrap_or_default() + "\n")
            });
            if let Err(error) = written {
                eprintln!("{case_id}: writing {}'s observation: {error}", system.id);
            }
        }
        runs.insert(&system.id, run);
    }

    let systems: serde_json::Map<String, Json> = runs
        .iter()
        .map(|(id, run)| {
            let record = match run {
                Ok(run) => json!({ "end": run.observation.end.name(), "events": run.observation.log.len(), "cost": run.cost }),
                Err(reason) => json!({ "error": reason }),
            };
            ((*id).to_owned(), record)
        })
        .collect();
    let pairs: Vec<Json> = manifest
        .pair
        .iter()
        .map(|pair| {
            let verdict = match (runs.get(pair.left.as_str()), runs.get(pair.right.as_str())) {
                (Some(Ok(left)), Some(Ok(right))) => compare(comparator, &left.observation, &right.observation, pair),
                (None, _) | (_, None) => Err("the pair names a system the manifest does not".to_owned()),
                _ => Err("a side has no observation".to_owned()),
            };
            let mut record = json!({
                "left": pair.left, "right": pair.right, "tier": pair.tier, "normalizations": pair.normalizations,
            });
            match verdict {
                Ok(verdict) => record["verdict"] = json!(verdict),
                Err(reason) => record["error"] = json!(reason),
            }
            record
        })
        .collect();
    json!({ "case": case_id, "systems": systems, "pairs": pairs })
}

fn main() {
    let mut args = env::args().skip(1);
    let (mut manifest_path, mut definition_ir, mut compare_ir) = (None, None, None);
    let (mut observations, mut report) = (None, None);
    let mut only = Vec::new();
    while let Some(arg) = args.next() {
        let mut value = || PathBuf::from(args.next().unwrap_or_else(|| usage()));
        match arg.as_str() {
            "--corpus" => manifest_path = Some(value()),
            "--definition-ir" => definition_ir = Some(value()),
            "--compare-ir" => compare_ir = Some(value()),
            "--observations" => observations = Some(value()),
            "--report" => report = Some(value()),
            _ if arg.starts_with("--") => usage(),
            _ => only.push(arg),
        }
    }
    let (Some(manifest_path), Some(definition_ir), Some(compare_ir)) = (manifest_path, definition_ir, compare_ir) else {
        usage()
    };
    let manifest: Manifest = toml::from_str(&fs::read_to_string(&manifest_path).expect("manifest")).expect("manifest TOML");
    assert_eq!(manifest.schema_version, 1, "manifest schema_version");
    let root = manifest_path.parent().unwrap_or(Path::new(".")).to_owned();
    let mut cases = manifest.cases.clone().unwrap_or_else(|| {
        let mut ids: Vec<String> = fs::read_dir(root.join("cases"))
            .expect("cases/")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        ids.sort();
        ids
    });
    if !only.is_empty() {
        cases.retain(|id| only.contains(id));
    }

    let (definition, stage) = load(&definition_ir);
    let (compare_definition, compare_stage) = load(&compare_ir);
    let context = Context { manifest: &manifest, root: &root, observations: observations.as_deref() };
    let results = Mutex::new(Vec::new());
    thread::scope(|scope| {
        for case_id in &cases {
            let (definition, stage, compare_definition, compare_stage) =
                (&definition, stage.clone(), &compare_definition, compare_stage.clone());
            let (context, results) = (&context, &results);
            thread::Builder::new()
                .stack_size(256 << 20)
                .spawn_scoped(scope, move || {
                    let mut semantics =
                        Semantics::from_stage(definition, stage, None, &mut Timings::disabled()).expect("load stage");
                    let mut comparator = Semantics::from_stage(compare_definition, compare_stage, None, &mut Timings::disabled())
                        .expect("load comparator stage");
                    let record = run_case(context, &mut semantics, &mut comparator, case_id);
                    let mut line = format!("{case_id}:");
                    if let Some(error) = record.get("error") {
                        line += &format!(" error {error}");
                    }
                    for pair in record["pairs"].as_array().into_iter().flatten() {
                        let outcome = pair.get("verdict").or(pair.get("error")).cloned().unwrap_or_default();
                        line += &format!(" {}~{}@{}: {}", pair["left"], pair["right"], pair["tier"], outcome);
                    }
                    println!("{}", line.replace('"', ""));
                    results.lock().expect("results").push(record);
                })
                .expect("case thread");
        }
    });
    let mut results = results.into_inner().expect("results");
    results.sort_by(|a, b| a["case"].as_str().cmp(&b["case"].as_str()));
    if let Some(path) = report {
        let report = json!({ "schema_version": 1, "manifest": manifest_path, "cases": results });
        fs::write(path, serde_json::to_string_pretty(&report).expect("report") + "\n").expect("write report");
    }
}
