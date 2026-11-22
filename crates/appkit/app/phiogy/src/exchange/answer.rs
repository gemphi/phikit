//! An answer: what the exchange writes back. Its first line and every
//! line of its head come from `data/head.tsv` and `data/status.tsv`;
//! what a refusal says from `data/fault.tsv`.

use serde_json::{Map, Value};

use super::set::{CORS, COOKIE, JSON, LENGTH, Set};
use crate::comp::Fault;

/// Who an answer is for: the page's origin when it may call from a
/// browser, and a new cookie's value when the page was given one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct To<'a> {
    pub origin: Option<&'a str>,
    pub cookie: Option<&'a str>,
}

/// The head of an answer of that status: its first line, the lines of
/// its kinds, and the lines for the page it is for. The empty line that
/// ends a head is not in it.
pub fn head(set: &Set, status: u16, kind: &[&str], to: To) -> String {
    let mut out = format!("{} {status} {}\r\n", set.speak, set.status(status));
    let mut add = |line: String| out.push_str(&format!("{line}\r\n"));
    kind.iter().flat_map(|kind| set.head(kind)).for_each(|line| add(line.to_string()));
    if let Some(origin) = to.origin {
        set.head(CORS).for_each(|line| add(line.replace("{origin}", origin)));
    }
    if let Some(value) = to.cookie {
        set.head(COOKIE).for_each(|line| add(line.replace("{name}", &set.cookie).replace("{value}", value)));
    }
    out
}

/// A whole answer with a body, which may be empty.
pub fn whole(set: &Set, status: u16, kind: &[&str], body: &str, to: To) -> Vec<u8> {
    let mut out = head(set, status, kind, to);
    set.head(LENGTH).for_each(|line| out.push_str(&format!("{}\r\n", line.replace("{length}", &body.len().to_string()))));
    format!("{out}\r\n{body}").into_bytes()
}

/// An answer in JSON.
pub fn json(set: &Set, status: u16, body: &Value, to: To) -> Vec<u8> {
    whole(set, status, &[JSON], &body.to_string(), to)
}

/// A refusal: the fault's status, its word and its sentence.
pub fn fail(set: &Set, fault: Fault, to: To) -> Vec<u8> {
    about(set, fault, "", to)
}

/// A refusal whose sentence names what it is about.
pub fn about(set: &Set, fault: Fault, key: &str, to: To) -> Vec<u8> {
    let (status, word, line) = set.refusal.of(fault);
    let line = line.replace("{key}", key);
    let body = Map::from_iter([(set.wire.error.clone(), Value::String(word.to_string())), (set.wire.message.clone(), Value::String(line))]);
    json(set, status, &Value::Object(body), to)
}
