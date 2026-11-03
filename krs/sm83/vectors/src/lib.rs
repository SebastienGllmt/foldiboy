//! Shared support for the harnesses: loading the definition, building
//! configuration values, and reading result cells by their KRS names.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use krs_exec::{
    CompiledStage, Decl, Halt, RunOptions, Semantics, State, StateView, StopRules, Term, Value, ValueIndex,
};
use krs_ir::measure::Timings;
use krs_ir_levels::DefinitionIr;

pub fn load(path: &Path) -> (DefinitionIr, CompiledStage) {
    let definition: DefinitionIr =
        krs_ir::json::from_slice(&fs::read(path).expect("definition IR")).expect("definition IR JSON");
    let stage = CompiledStage::compile(&definition, &mut Timings::disabled()).expect("compile");
    (definition, stage)
}

/// The 8-bit registers, as the corpus spells them.
pub const REGISTERS: [&str; 7] = ["a", "b", "c", "d", "e", "h", "l"];

/// One event of the bus log (the observation contract's vocabulary, `sm83-events`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Read { addr: i64, value: i64 },
    Write { addr: i64, value: i64 },
    Idle,
}

/// The declarations the harnesses build and read.
pub struct Names {
    index: Arc<ValueIndex>,
    registers: [Value; 7],
    read: Decl,
    write: Decl,
    idle: Decl,
}

impl Names {
    pub fn new(index: &Arc<ValueIndex>) -> Self {
        let decl = |name: &str| index.decl(name).unwrap_or_else(|error| panic!("{name}: {error}"));
        Self {
            index: index.clone(),
            registers: REGISTERS.map(|name| {
                let name = format!("Register::{}", name.to_uppercase());
                index.ctor(&name, []).unwrap_or_else(|error| panic!("{name}: {error}"))
            }),
            read: decl("Event::Read"),
            write: decl("Event::Write"),
            idle: decl("Event::Idle"),
        }
    }

    /// The constructor `name` applied to `fields`.
    pub fn ctor(&self, name: &str, fields: Vec<(&str, Value)>) -> Value {
        self.index.ctor(name, fields).unwrap_or_else(|error| panic!("{name}: {error}"))
    }

    /// The flags of a flag byte, which holds Z, N, H, C in bits 7 to 4.
    pub fn flags(&self, byte: i64) -> Value {
        let bit = |weight: i64| Value::from(byte & weight != 0);
        self.ctor("Flags", vec![("z", bit(0x80)), ("n", bit(0x40)), ("h", bit(0x20)), ("c", bit(0x10))])
    }

    /// The flag byte of a result's flags, Z, N, H, C in bits 7 to 4.
    pub fn flag_byte(&self, view: &StateView) -> Result<i64, String> {
        let value = cell(view, "f")?;
        let flags = value.as_ctor().ok_or("<f> is not a constructor")?;
        let bit = |name: &str, weight: i64| {
            let set = flags.field(name).and_then(Value::as_bool).ok_or(format!("{name} of <f>"))?;
            Ok::<i64, String>(if set { weight } else { 0 })
        };
        Ok(bit("z", 0x80)? | bit("n", 0x40)? | bit("h", 0x20)? | bit("c", 0x10)?)
    }

    /// The register file, from each register's value in `REGISTERS` order.
    pub fn register_file(&self, values: [i64; 7]) -> Value {
        Value::map(self.registers.iter().cloned().zip(values.map(Value::from)))
    }

    /// The register file of a result, keyed as in `REGISTERS`.
    pub fn registers(&self, view: &StateView) -> Result<BTreeMap<String, i64>, String> {
        let file = cell(view, "regs")?;
        let file = file.as_map().ok_or("<regs> is not a map")?;
        REGISTERS
            .iter()
            .zip(&self.registers)
            .map(|(name, key)| {
                let value = file.get(key).and_then(Value::as_i64).ok_or(format!("register {name}"))?;
                Ok(((*name).to_owned(), value))
            })
            .collect()
    }

    /// The bus log of a result.
    pub fn bus(&self, view: &StateView) -> Result<Vec<Access>, String> {
        let log = cell(view, "bus")?;
        let items = log.as_list().ok_or("<bus> is not a list")?;
        items
            .iter()
            .map(|item| {
                let cycle = item.as_ctor().ok_or(format!("bus entry {item:?}"))?;
                let field = |name: &str| {
                    cycle.field(name).and_then(Value::as_i64).ok_or(format!("{name} of {item:?}"))
                };
                let decl = cycle.decl();
                if *decl == self.read {
                    Ok(Access::Read { addr: field("addr")?, value: field("value")? })
                } else if *decl == self.write {
                    Ok(Access::Write { addr: field("addr")?, value: field("value")? })
                } else if *decl == self.idle {
                    Ok(Access::Idle)
                } else {
                    Err(format!("unknown bus entry {decl}"))
                }
            })
            .collect()
    }
}

/// The value of a leaf cell.
pub fn cell(view: &StateView, path: &str) -> Result<Value, String> {
    view.value(path).map_err(|error| error.to_string())
}

/// An `Int` cell.
pub fn int(view: &StateView, path: &str) -> Result<i64, String> {
    cell(view, path)?.as_i64().ok_or(format!("<{path}> is not an Int"))
}

/// A `Bool` cell.
pub fn bool(view: &StateView, path: &str) -> Result<bool, String> {
    cell(view, path)?.as_bool().ok_or(format!("<{path}> is not a Bool"))
}

/// A map from `Int` to `Int`, such as `memory`.
pub fn int_map(view: &StateView, path: &str) -> Result<BTreeMap<i64, i64>, String> {
    let map = cell(view, path)?;
    let entries = map.as_map().ok_or(format!("<{path}> is not a map"))?;
    entries
        .iter()
        .map(|(key, value)| Ok((key.as_i64().ok_or("address")?, value.as_i64().ok_or("byte")?)))
        .collect()
}

/// The rules that rewrite `#boundary`: one of them starts every instruction step.
pub const BOUNDARY_RULES: [&str; 4] = ["boundary-dispatch", "boundary-fetch", "boundary-wake", "boundary-halted"];

/// The state one boundary step after `state`, a state at an instruction boundary.
///
/// The machine runs forever, so a step ends before the next boundary rule: a run
/// from `state` stops before the first one (krs-exec stop rules), and a run resumed
/// from there takes that rule's step and stops before the next. `None` when no
/// boundary rule is reached within `depth` rewrite steps.
pub fn next_boundary(
    semantics: &mut Semantics,
    state: &State,
    depth: u64,
) -> Result<Option<Term>, String> {
    let options = RunOptions {
        depth: Some(depth),
        stop: StopRules { before: BOUNDARY_RULES.iter().map(|rule| (*rule).to_owned()).collect(), after: vec![] },
        ..RunOptions::default()
    };
    let mut report = semantics.run(state, options.clone()).map_err(|error| format!("run: {error}"))?;
    if report.halt == Halt::Stopped && report.steps == 0 {
        report = semantics.run(&report.resume(), options).map_err(|error| format!("run: {error}"))?;
    }
    Ok((report.halt == Halt::Stopped && report.steps > 0).then_some(report.state))
}
