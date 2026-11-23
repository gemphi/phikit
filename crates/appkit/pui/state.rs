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
