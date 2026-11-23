//! What Phiogy remembers, behind appkit's three functions: get, set
//! and has, by path. A path's first step names the set, the rest is
//! the key in it:
//!
//! * `choice/NAME`: what the person chose (which look, which room,
//!   which tab), kept with their other choices.
//! * any other first step is one of the person's own documents by the
//!   name `data/keep.tsv` gives it (`star`, `seen`, `like`, `follow`),
//!   and the rest is the key in that document: `star/phiogy/lobby`.
//!
//! A flag is `1` or nothing; a number is written out.

use appkit::pui::state::{ON, PState, step};
use serde_json::{Value, json};

use super::Desk;

/// The set of what the person chose.
pub const CHOICE: &str = "choice";

impl PState for Desk {
    fn get(&self, path: &str) -> String {
        let (set, key) = step(path);
        match set {
            CHOICE => self.choice.get(key),
            _ => match self.kept.get(set).get(key) {
                Some(Value::Bool(true)) => ON.to_string(),
                Some(Value::String(text)) => text.clone(),
                Some(Value::Number(number)) => number.to_string(),
                _ => String::new(),
            },
        }
    }

    fn set(&self, path: &str, value: &str) {
        let (set, key) = step(path);
        let kept = match (value, value.parse::<f64>()) {
            ("", _) => json!(false),
            (ON, _) => json!(true),
            (_, Ok(number)) => json!(number),
            (text, Err(_)) => json!(text),
        };
        match set {
            CHOICE => self.choice.put(key, value),
            _ => self.kept.put(set, key, kept),
        }
    }
}
