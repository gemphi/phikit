//! Controls: the small things a part is worked with, each an icon that
//! says what it is called while the pointer is over it (the kit's
//! tip). Which controls a part has, the icon of each and the row of
//! `data/text.tsv` that names it are a table: `data/tool.tsv` for the
//! viewer, `data/act.tsv` for a room. A row whose name ends in `.on`
//! is the same control once it is on.

use appkit::service::PTable;

use crate::data;

/// How the name of a control that is on ends.
pub const ON: &str = ".on";

/// A table of controls.
#[derive(Debug, Clone, Default)]
pub struct Tool {
    /// A name, its icon, the row of the words that names it.
    row: Vec<(String, String, String)>,
}

impl Tool {
    /// The controls of a table.
    pub fn read(table: &str) -> Tool {
        let row = data::row(table).into_iter().filter_map(|cell| Some((cell.first()?.to_string(), cell.get(1)?.to_string(), cell.get(2)?.to_string())));
        Tool { row: row.collect() }
    }

    /// The names of its controls, in the order of their rows.
    pub fn name(&self) -> impl Iterator<Item = &str> {
        self.row.iter().map(|(name, ..)| name.as_str())
    }

    fn row<'a>(&'a self, name: &str, text: &'a PTable) -> Option<(&'a str, &'a str)> {
        self.row.iter().find(|(known, ..)| known == name).map(|(_, icon, word)| (icon.as_str(), text.get(word)))
    }

    /// The icon and the word of a control: of the row for it on, when
    /// it is on and has one. Nothing for one the table leaves out.
    pub fn of<'a>(&'a self, name: &str, on: bool, text: &'a PTable) -> Option<(&'a str, &'a str)> {
        Some(format!("{name}{ON}")).filter(|_| on).and_then(|name| self.row(&name, text)).or_else(|| self.row(name, text))
    }
}

/// A number as a person reads it: its thousands apart, as the web
/// writes them.
pub fn grouped(number: u64) -> String {
    let digit = number.to_string();
    let part: Vec<&str> = digit.as_bytes().rchunks(3).rev().filter_map(|part| std::str::from_utf8(part).ok()).collect();
    part.join(",")
}
