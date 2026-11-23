//! State: everything an app remembers and everything its parts are
//! driven by, behind three functions: get, set, has. What is meant is
//! said by a path: its first step names the set (which document, which
//! table), and the rest is the key in that set. `star/phiogy/lobby` is
//! the key `phiogy/lobby` of the set `star`.
//!
//! An app says where each set is kept; a part asks by path and knows
//! nothing of files, tables or servers. A value is text: a flag is the
//! text `1` or nothing, a number is written out.

/// What an app remembers, by path.
pub trait PState {
    /// What is kept at a path; nothing for a path with nothing.
    fn get(&self, path: &str) -> String;

    /// Keep something at a path. Nothing takes it away.
    fn set(&self, path: &str, value: &str);

    /// Whether a path has something.
    fn has(&self, path: &str) -> bool {
        !self.get(path).is_empty()
    }
}

/// What a flag that is set is written as.
pub const ON: &str = "1";

/// A path's set and its key in that set: the first step, and the rest.
/// A path of one step is a set with no key.
pub fn step(path: &str) -> (&str, &str) {
    path.split_once('/').unwrap_or((path, ""))
}

/// What a flag is written as: `ON`, or nothing.
pub fn flag(on: bool) -> &'static str {
    match on {
        true => ON,
        false => "",
    }
}

/// State kept in memory for as long as the window is open: what only a
/// window needs (which tab is open), its first values from a table of
/// `path value` rows.
#[derive(Debug, Default)]
pub struct PMemory(std::cell::RefCell<std::collections::HashMap<String, String>>);

impl PMemory {
    pub fn read(table: &str) -> PMemory {
        let row = table.lines().filter(|l| !l.starts_with('#')).filter_map(|l| l.split_once('\t'));
        PMemory(std::cell::RefCell::new(row.map(|(p, v)| (p.to_string(), v.to_string())).collect()))
    }
}

impl PState for PMemory {
    fn get(&self, path: &str) -> String {
        self.0.borrow().get(path).cloned().unwrap_or_default()
    }

    fn set(&self, path: &str, value: &str) {
        self.0.borrow_mut().insert(path.to_string(), value.to_string());
    }
}

/// State kept by something else, reached through two functions: what an
/// app's world answers by path, handed over without a type of its own.
pub struct PFn {
    pub get: Box<dyn Fn(&str) -> String>,
    pub set: PSetter,
}

/// What keeps a value at a path, for `PFn`.
pub type PSetter = Box<dyn Fn(&str, &str)>;

impl PState for PFn {
    fn get(&self, path: &str) -> String {
        (self.get)(path)
    }

    fn set(&self, path: &str, value: &str) {
        (self.set)(path, value)
    }
}
