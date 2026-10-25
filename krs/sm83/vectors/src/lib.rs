//! Shared support for the harnesses: loading the definition, building
//! configuration values, and reading result cells by their KRS names.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use krs_exec::{CompiledStage, Decl, StateView, Value, ValueIndex};
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

/// One entry of the bus log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Read { addr: i64, value: i64 },
    Write { addr: i64, old: i64, new: i64 },
    Internal,
}

/// The declarations the harnesses build and read.
pub struct Names {
    index: Arc<ValueIndex>,
    registers: [Value; 7],
    read: Decl,
    write: Decl,
    internal: Decl,
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
            read: decl("Cycle::Read"),
            write: decl("Cycle::Write"),
            internal: decl("Cycle::Internal"),
        }
    }

    /// The constructor `name` applied to `fields`.
    pub fn ctor(&self, name: &str, fields: Vec<(&str, Value)>) -> Value {
        self.index.ctor(name, fields).unwrap_or_else(|error| panic!("{name}: {error}"))
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
                    Ok(Access::Write { addr: field("addr")?, old: field("old")?, new: field("new")? })
                } else if *decl == self.internal {
                    Ok(Access::Internal)
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

/// Succeeds when the computation cell is empty; otherwise reports its start.
pub fn finished(view: &StateView) -> Result<(), String> {
    let k = cell(view, "k")?;
    match k.as_seq() {
        Some([]) => Ok(()),
        _ => {
            let text = format!("{k:?}");
            Err(format!("<k> is {}", &text[..text.len().min(1500)]))
        }
    }
}
