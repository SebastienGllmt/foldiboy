//! The observation of one run (draft-personal/observer-design/design.md): how it ended, its
//! event log and its end state, as every system adapter produces it.
//!
//! The JSON form is the typed input encoding of `sm83-observe`'s `Observation` sort, the
//! encoding the comparator (`sm83-compare`) and an external system's observation file use.

use std::collections::BTreeMap;

use krs_exec::StateView;
use serde_json::{Value as Json, json};

use crate::{Access, Names, bool as cell_bool, cell, int as cell_int, int_map};

/// How a run ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndClass {
    /// The CPU executed an unused opcode: the end a corpus case is written for.
    Locked,
    /// No step applies, other than the lock: the run left what the system covers.
    Stuck,
    /// The system's budget ran out first.
    Bound,
}

impl EndClass {
    pub fn name(self) -> &'static str {
        match self {
            Self::Locked => "Locked",
            Self::Stuck => "Stuck",
            Self::Bound => "Bound",
        }
    }

    fn parse(name: &str) -> Result<Self, String> {
        match name {
            "Locked" => Ok(Self::Locked),
            "Stuck" => Ok(Self::Stuck),
            "Bound" => Ok(Self::Bound),
            _ => Err(format!("unknown end class {name}")),
        }
    }
}

/// The architectural state when the run ended: what a program can read back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndState {
    /// A B C D E H L, in that order.
    pub registers: [i64; 7],
    /// Z N H C.
    pub flags: [bool; 4],
    pub pc: i64,
    pub sp: i64,
    pub ime: bool,
    pub ei_pending: bool,
    pub halted: bool,
    pub halt_bug: bool,
    /// Every byte outside the ROM region the run's memory holds, by address.
    pub memory: BTreeMap<i64, i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub end: EndClass,
    pub log: Vec<Access>,
    pub state: EndState,
}

const REGISTER_FIELDS: [&str; 7] = ["a", "b", "c", "d", "e", "h", "l"];
const FLAG_FIELDS: [&str; 4] = ["zf", "nf", "hf", "cf"];

impl Observation {
    /// The observation in the typed input encoding of `Observation`.
    pub fn to_json(&self) -> Json {
        let log: Vec<Json> = self
            .log
            .iter()
            .map(|access| match *access {
                Access::Read { addr, value } => json!({ "Event::Read": { "addr": addr, "value": value } }),
                Access::Write { addr, value } => json!({ "Event::Write": { "addr": addr, "value": value } }),
                Access::Idle => json!({ "Event::Idle": [] }),
            })
            .collect();
        let mut state = serde_json::Map::new();
        for (name, value) in REGISTER_FIELDS.iter().zip(self.state.registers) {
            state.insert((*name).to_owned(), json!(value));
        }
        for (name, value) in FLAG_FIELDS.iter().zip(self.state.flags) {
            state.insert((*name).to_owned(), json!(value));
        }
        let s = &self.state;
        state.insert("pc".to_owned(), json!(s.pc));
        state.insert("sp".to_owned(), json!(s.sp));
        state.insert("ime".to_owned(), json!(s.ime));
        state.insert("ei_pending".to_owned(), json!(s.ei_pending));
        state.insert("halted".to_owned(), json!(s.halted));
        state.insert("halt_bug".to_owned(), json!(s.halt_bug));
        let memory: Vec<Json> = s.memory.iter().map(|(addr, byte)| json!([addr, byte])).collect();
        state.insert("memory".to_owned(), json!({ "$map": memory }));
        json!({ "Run": { "end": self.end.name(), "log": { "$list": log }, "state": { "State": state } } })
    }

    /// The observation `to_json` writes, read back: an external system's observation file.
    pub fn from_json(document: &Json) -> Result<Self, String> {
        let run = document.get("Run").ok_or("an observation is {\"Run\": {end, log, state}}")?;
        let end = EndClass::parse(run.get("end").and_then(Json::as_str).ok_or("end")?)?;
        let entries = run.pointer("/log/$list").and_then(Json::as_array).ok_or("log is {\"$list\": [..]}")?;
        let log = entries
            .iter()
            .map(|entry| {
                let int = |event: &str, field: &str| {
                    entry.pointer(&format!("/{event}/{field}")).and_then(Json::as_i64).ok_or(format!("{field} of {entry}"))
                };
                if entry.get("Event::Read").is_some() {
                    Ok(Access::Read { addr: int("Event::Read", "addr")?, value: int("Event::Read", "value")? })
                } else if entry.get("Event::Write").is_some() {
                    Ok(Access::Write { addr: int("Event::Write", "addr")?, value: int("Event::Write", "value")? })
                } else if entry.get("Event::Idle").is_some() {
                    Ok(Access::Idle)
                } else {
                    Err(format!("unknown event {entry}"))
                }
            })
            .collect::<Result<_, String>>()?;
        let state = run.pointer("/state/State").ok_or("state is {\"State\": {..}}")?;
        let int = |name: &str| state.get(name).and_then(Json::as_i64).ok_or(format!("state field {name}"));
        let bool = |name: &str| state.get(name).and_then(Json::as_bool).ok_or(format!("state field {name}"));
        let mut registers = [0; 7];
        for (slot, name) in registers.iter_mut().zip(REGISTER_FIELDS) {
            *slot = int(name)?;
        }
        let mut flags = [false; 4];
        for (slot, name) in flags.iter_mut().zip(FLAG_FIELDS) {
            *slot = bool(name)?;
        }
        let memory = state
            .pointer("/memory/$map")
            .and_then(Json::as_array)
            .ok_or("memory is {\"$map\": [[addr, byte], ..]}")?
            .iter()
            .map(|pair| {
                let pair = pair.as_array().filter(|pair| pair.len() == 2).ok_or("a memory entry is [addr, byte]")?;
                Ok((pair[0].as_i64().ok_or("address")?, pair[1].as_i64().ok_or("byte")?))
            })
            .collect::<Result<_, String>>()?;
        Ok(Self {
            end,
            log,
            state: EndState {
                registers,
                flags,
                pc: int("pc")?,
                sp: int("sp")?,
                ime: bool("ime")?,
                ei_pending: bool("ei_pending")?,
                halted: bool("halted")?,
                halt_bug: bool("halt_bug")?,
                memory,
            },
        })
    }
}

impl EndState {
    /// The end state of a configuration of the SM83 definition, read cell by cell.
    pub fn from_krs(names: &Names, view: &StateView) -> Result<Self, String> {
        let file = names.registers(view)?;
        let registers = REGISTER_FIELDS.map(|name| file[name]);
        let byte = names.flag_byte(view)?;
        let flags = [0x80, 0x40, 0x20, 0x10].map(|weight| byte & weight != 0);
        Ok(Self {
            registers,
            flags,
            pc: cell_int(view, "pc")?,
            sp: cell_int(view, "sp")?,
            ime: cell_bool(view, "ime")?,
            ei_pending: cell_bool(view, "ei_pending")?,
            halted: cell_bool(view, "halted")?,
            halt_bug: cell_bool(view, "halt_bug")?,
            memory: int_map(view, "memory")?,
        })
    }
}

/// The computation cell's head constructor, by its KRS name (`Step::Locked`, ...), for
/// deciding the end class of a stuck run.
pub fn k_head(view: &StateView) -> Result<String, String> {
    let k = cell(view, "k")?;
    let head = k.as_seq().and_then(<[_]>::first).ok_or(format!("<k> is {k:?}"))?;
    head.as_ctor().map(|ctor| ctor.decl().to_string()).ok_or(format!("<k> head {head:?}"))
}
