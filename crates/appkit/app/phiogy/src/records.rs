//! Every record Phiogy keeps on Phiora, as one small thing: a few words and
//! a payload. A record is of a **kind** (`club`, `host`, `message`, `bond`,
//! `mark`, `follow`) and has a **name key**, the `ref` its maker gave it
//! (`m1764000000000`, `fl:<me>><them>`); its payload is its fields.
//!
//! One function does the work, `Records::act`: it is told what is asked
//! (`Ask`) and answers with what came of it (`Reply`), by a `match`. The
//! three that are used are its words: `has`, `get` and `set`.
//!
//! * `has(kind, key)`: whether Phiora has it.
//! * `get(kind, key)`: the record, its payload as Phiora has it.
//! * `set(kind, key, payload)`: make it, or change it to match: only the
//!   fields that differ are sent, so the same payload again is `Same`.
//!
//! Who may change a record is Phiora's to say (its maker, the person it
//! names in `userId`, an editor); a guest is refused there. What each ask is
//! called, and every word said of one, is `data/make.tsv`.
use appkit::service::PTable;
use serde_json::{Map, Value, json};

use crate::data;
use crate::make::enc;
use crate::phiora::{Phiora, Verb};

/// What is asked of a record.
#[derive(Debug, Clone, PartialEq)]
pub enum Ask {
    /// Whether Phiora has it.
    Has,
    /// It, as Phiora has it.
    Get,
    /// Make it, or change it to match this (an object of fields).
    Set(Value),
}

/// What came of setting a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    /// There was none: it is made, and this is the id Phiora gave it.
    Made(String),
    /// There was one: these fields were changed.
    Changed(Vec<String>),
    /// There was one, already as the payload says.
    Same,
}

/// What came of an ask, one for each.
#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    Has(bool),
    Got(Option<Value>),
    Done(Done),
}

impl Done {
    /// What was done, in the words of `data/make.tsv` (`made club x (id)`).
    pub fn said(&self, kind: &str, key: &str) -> String {
        let word = PTable::read(data::MAKE);
        let line = |name: &str| word.fill(name, "kind", kind).replace("{ref}", key);
        match self {
            Done::Made(id) => line("say.made").replace("{id}", id),
            Done::Changed(fields) => line("say.changed").replace("{fields}", &fields.join(", ")),
            Done::Same => line("say.same"),
        }
    }
}

/// Phiora's records, asked and told by kind and name key.
#[derive(Debug, Clone)]
pub struct Records {
    phiora: Phiora,
    word: PTable,
}

impl Records {
    pub fn new(phiora: Phiora) -> Records {
        Records { phiora, word: PTable::read(data::MAKE) }
    }

    /// Ask for a record, by its kind and name key, and say what came of it.
    /// `Err` is why not, in Phiora's words where it gave any, else that it
    /// did not answer.
    pub fn act(&self, kind: &str, key: &str, ask: Ask) -> Result<Reply, String> {
        let word = &self.word;
        let nowhere = || word.get("say.nowhere").to_string();
        // What Phiora has under that key: asked for that one, and narrowed again here, for a Phiora that sends all.
        let path = format!("{}&where={}:{}", word.fill("list", "kind", kind), word.get("column.ref"), enc(key));
        let tree: Value = serde_json::from_str(&self.phiora.get(&path).ok_or_else(nowhere)?).map_err(|_| nowhere())?;
        let sources = tree[word.get("answer.sources")].as_array().ok_or_else(nowhere)?;
        let rows = sources.iter().filter(|source| source[word.get("answer.kind")] == kind).filter_map(|source| source[word.get("answer.rows")].as_array()).flatten();
        let had = rows.filter(|row| row[word.get("column.ref")] == key).find_map(|row| row.as_object().cloned());
        // Phiora's refusal, in its own words if it said any.
        let refused = |said: Option<(bool, String)>| match said {
            Some((_, text)) => serde_json::from_str::<Value>(&text).ok().and_then(|said| said[word.get("answer.message")].as_str().map(str::to_string)).unwrap_or(text),
            None => nowhere(),
        };
        match (ask, had) {
            (Ask::Has, record) => Ok(Reply::Has(record.is_some())),
            (Ask::Get, record) => Ok(Reply::Got(record.map(Value::Object))),
            (Ask::Set(payload), record) => {
                let mut fields = payload.as_object().cloned().unwrap_or_default();
                fields.insert(word.get("column.ref").to_string(), json!(key));
                match record {
                    // There is one: only what differs is sent.
                    Some(record) => {
                        let set: Map<String, Value> = fields.into_iter().filter(|(name, value)| record.get(name) != Some(value)).collect();
                        let id = record.get(word.get("id")).and_then(Value::as_str).unwrap_or_default();
                        match (set.is_empty(), self.phiora.record(Verb::Patch, &word.get("item").replace("{kind}", kind).replace("{id}", id), &json!({ "set": set }).to_string())) {
                            (true, _) => Ok(Reply::Done(Done::Same)),
                            (false, Some((true, _))) => Ok(Reply::Done(Done::Changed(set.keys().cloned().collect()))),
                            (false, other) => Err(refused(other)),
                        }
                    }
                    // There is none: it is made.
                    None => match self.phiora.record(Verb::Post, word.get("tree"), &json!({ "kind": kind, "fields": fields }).to_string()) {
                        Some((true, text)) => Ok(Reply::Done(Done::Made(serde_json::from_str::<Value>(&text).ok().and_then(|said| said[word.get("answer.item")][word.get("id")].as_str().map(str::to_string)).unwrap_or_default()))),
                        other => Err(refused(other)),
                    },
                }
            }
        }
    }

    /// Whether Phiora has the record. Not knowing is not having.
    pub fn has(&self, kind: &str, key: &str) -> bool {
        matches!(self.act(kind, key, Ask::Has), Ok(Reply::Has(true)))
    }

    /// The record, its payload as Phiora has it (with its `id`); `None` for
    /// one it has not.
    pub fn get(&self, kind: &str, key: &str) -> Result<Option<Value>, String> {
        match self.act(kind, key, Ask::Get)? {
            Reply::Got(record) => Ok(record),
            _ => Ok(None),
        }
    }

    /// Make the record, or change it to match the payload.
    pub fn set(&self, kind: &str, key: &str, payload: Value) -> Result<Done, String> {
        match self.act(kind, key, Ask::Set(payload))? {
            Reply::Done(done) => Ok(done),
            _ => Ok(Done::Same),
        }
    }
}
