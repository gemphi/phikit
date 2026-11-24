//! Prim names: any name made a USD identifier and kept unique among its
//! siblings, and the names the writer gives its own prims (`data/usd.tsv`).

use crate::table::TsvTable;
use std::collections::HashSet;
use std::sync::OnceLock;

/// The names already given under one parent; each new one is cleaned and,
/// when taken, numbered (`Cube`, `Cube_1`, `Cube_2` …).
#[derive(Clone, Debug, Default)]
pub struct UsdNameSet {
    taken: HashSet<String>,
}

impl UsdNameSet {
    /// `raw` as a USD identifier no sibling has.
    pub fn take(&mut self, raw: &str) -> String {
        let clean = Self::clean(raw);
        let mut name = clean.clone();
        let mut n = 0;
        while !self.taken.insert(name.clone()) {
            n += 1;
            name = format!("{clean}_{n}");
        }
        name
    }

    /// `raw` as a USD identifier: letters, digits and `_`, not starting
    /// with a digit, never empty.
    pub fn clean(raw: &str) -> String {
        let mut name: String = raw
            .chars()
            .map(|c| match c.is_ascii_alphanumeric() {
                true => c,
                false => '_',
            })
            .collect();
        match name.chars().next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
            _ => name.insert(0, '_'),
        }
        name
    }
}

/// A name the writer gives a prim of its own: `root`, `look` (the
/// materials scope), `shader`, `up` (the stage's up axis).
pub fn usd_word(key: &str) -> &'static str {
    static TABLE: OnceLock<TsvTable> = OnceLock::new();
    let table = TABLE.get_or_init(|| TsvTable::parse(include_str!("../../../data/usd.tsv")));
    table.lookup("key", key, "value").unwrap_or("")
}
