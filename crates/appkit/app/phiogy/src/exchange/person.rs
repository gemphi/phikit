//! What a page asks of the line besides a room's call, answered as
//! Phiora answers it (`puikit/app/phiora-server`), so the web Phiogy
//! pointed at the line gets its config and keeps a person's documents
//! there:
//!
//! * `GET /app/KEY`: the app's own document, read from the app's file
//!   each time it is asked for, so the line and the web have one source.
//! * `GET /me`: who the page belongs to.
//! * `GET` and `PUT /state/NAME`: a document the person keeps, such as
//!   which rooms they follow.
//!
//! The paths, the file and the names in `me` are `data/serve.tsv`.
//! Documents are kept while the line runs, not on disk; and the ones
//! Phiora writes itself (the wallet, what is owned) are not here.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use appkit::service::PTable;
use serde_json::{Map, Value, json};

use super::ask::Ask;
use crate::board::lock;
use crate::comp::{APP, Fault, Limit};
use crate::data;
use crate::entity::Caller;

/// The kind of name a saved document has: its rule is `data/name.tsv`'s.
pub const STATE: &str = "state";
/// The methods these routes know.
const GET: &str = "GET";
const PUT: &str = "PUT";

/// What a page asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    /// The document of the app of that key.
    App(String),
    Me,
    /// The saved document of that name.
    State(String),
}

/// What is answered: a status and a body, or a fault and what its
/// sentence is about.
pub type Answer = Result<(u16, Option<Value>), (Fault, String)>;

/// The line's side of a person: the apps' files, and what people keep.
pub struct Person {
    word: PTable,
    folder: PathBuf,
    /// Each person's documents, by who they are and then by name.
    kept: Mutex<HashMap<String, HashMap<String, Value>>>,
}

/// A path's part with its `%XX` bytes read back.
fn plain(part: &str) -> String {
    let (mut out, mut rest) = (Vec::new(), part.as_bytes());
    while let Some((first, after)) = rest.split_first() {
        let byte = after.get(..2).and_then(|pair| std::str::from_utf8(pair).ok()).and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match (first, byte) {
            (b'%', Some(byte)) => {
                out.push(byte);
                rest = &after[2..];
            }
            _ => {
                out.push(*first);
                rest = after;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

impl Person {
    /// As `data/serve.tsv` says, with the apps' folder the system gives
    /// when it gives one.
    pub fn own() -> Person {
        let word = PTable::read(data::SERVE);
        let given = std::env::var(word.get("given")).ok().filter(|folder| !folder.trim().is_empty());
        let folder = given.map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(word.get("folder")), PathBuf::from);
        Person { word, folder, kept: Mutex::default() }
    }

    /// What a path asks for, if it is one of these. An older spelling
    /// of its first part answers as the one it stands for.
    pub fn asked(&self, path: &str) -> Option<Asked> {
        let (first, rest) = path.strip_prefix('/')?.split_once('/').map_or((path.trim_start_matches('/'), ""), |pair| pair);
        let alias = self.word.list().iter().filter(|(name, _)| name == "alias").filter_map(|(_, pair)| pair.split_once('\t')).find(|(old, _)| old.trim() == first);
        let first = alias.map_or(first, |(_, now)| now.trim());
        let word = |name: &str| self.word.get(name);
        match (first, rest.is_empty()) {
            (first, false) if first == word("app") => Some(Asked::App(rest.to_string())),
            (first, true) if first == word("me") => Some(Asked::Me),
            (first, false) if first == word("state") => Some(Asked::State(plain(rest))),
            _ => None,
        }
    }

    /// Who a page belongs to, as a page reads it. A guest has no handle.
    fn me(&self, caller: &Caller) -> Value {
        let word = |name: &str| self.word.get(name).to_string();
        let handle = caller.handle.clone().map_or(Value::Null, Value::String);
        let row = [(word("me.id"), json!(caller.user)), (word("me.name"), json!(caller.name)), (word("me.guest"), json!(caller.handle.is_none())), (word("me.handle"), handle), (word("me.mail"), Value::Null), (word("me.checked"), json!(false)), (word("me.demo"), json!(false))];
        Value::Object(Map::from_iter(row))
    }

    /// Answer what was asked, for whoever asks.
    pub fn answer(&self, asked: &Asked, ask: &Ask, caller: &Caller, limit: &Limit) -> Answer {
        let count = |name: &str| self.word.get(name).parse::<u16>().unwrap_or_default();
        let fault = |fault: Fault| (fault, String::new());
        match (asked, ask.method.as_str()) {
            (Asked::App(key), GET) => {
                let file = self.folder.join(self.word.get("file").replace("{key}", key));
                let read = Some(key).filter(|key| limit.fits(APP, key)).and_then(|_| std::fs::read_to_string(file).ok());
                let document = read.and_then(|text| serde_json::from_str::<Value>(&text).ok());
                document.map(|document| (count("read"), Some(document))).ok_or((Fault::App, key.clone()))
            }
            (Asked::Me, GET) => Ok((count("read"), Some(self.me(caller)))),
            (Asked::State(name), _) if !limit.fits(STATE, name) => Err(fault(Fault::Name)),
            (Asked::State(name), GET) => {
                let kept = lock(&self.kept).get(&caller.user).and_then(|own| own.get(name)).cloned();
                kept.map(|document| (count("read"), Some(document))).ok_or(fault(Fault::Unset))
            }
            (Asked::State(name), PUT) => {
                let document = Some(&ask.body).filter(|body| !body.is_empty()).ok_or(fault(Fault::Empty))?;
                let document = serde_json::from_slice::<Value>(document).map_err(|_| fault(Fault::Json))?;
                lock(&self.kept).entry(caller.user.clone()).or_default().insert(name.clone(), document);
                Ok((count("kept"), None))
            }
            _ => Err(fault(Fault::Route)),
        }
    }
}
