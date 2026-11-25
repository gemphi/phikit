//! A kit form's tabs read from a table of the app's data, as a thing's
//! page (`data/thing.tsv`) and the column beside a room
//! (`data/aside.tsv`) have them: a row `tab key icon lists` each, its
//! word `<prefix>.<key>` of data/text.tsv. A tab with nothing in it is
//! not shown; the one on is the one chosen where it holds something,
//! else the first that does; a tab's lists hold their rows only while it
//! is on. The rest of the form's lists are the page's own.

use appkit::pui::state::PState;

use super::Desk;
use crate::data;

type Row = Vec<String>;

/// The tabs of a table, and the path the one chosen is kept at.
pub struct Tabbed {
    pub table: &'static str,
    pub path: &'static str,
    /// The words' prefix: a tab's word is this, a dot, its key.
    pub prefix: &'static str,
}

impl Tabbed {
    /// Every tab of the table: its key, its icon, its lists.
    fn every(&self) -> Vec<(String, String, Vec<String>)> {
        data::row(self.table).into_iter().filter(|row| row.first() == Some(&"tab") && row.len() >= 4).map(|row| (row[1].to_string(), row[2].to_string(), row[3].split_whitespace().map(str::to_string).collect())).collect()
    }

    /// The tabs that hold something, by what the page's lists hold.
    pub fn shown(&self, list: &dyn Fn(&str) -> Vec<Row>) -> Vec<(String, String, Vec<String>)> {
        self.every().into_iter().filter(|(_, _, lists)| lists.iter().any(|name| !list(name).is_empty())).collect()
    }

    /// The tab on.
    pub fn on(&self, desk: &Desk, list: &dyn Fn(&str) -> Vec<Row>) -> String {
        let (tabs, kept) = (self.shown(list), desk.shelf.get(self.path));
        tabs.iter().find(|(key, ..)| *key == kept).or(tabs.first()).map(|(key, ..)| key.clone()).unwrap_or_default()
    }

    /// The tabs as the form's list: each its key, its word, its icon.
    pub fn list(&self, desk: &Desk, list: &dyn Fn(&str) -> Vec<Row>) -> Vec<Row> {
        self.shown(list).into_iter().map(|(key, icon, _)| vec![key.clone(), desk.text.get(&format!("{}.{key}", self.prefix)).to_string(), icon]).collect()
    }

    /// What a list holds now: a tab's lists only while it is on.
    pub fn rows(&self, desk: &Desk, name: &str, list: &dyn Fn(&str) -> Vec<Row>) -> Vec<Row> {
        let tab = self.every().into_iter().find(|(_, _, lists)| lists.iter().any(|one| one == name));
        match tab {
            Some((key, ..)) if key != self.on(desk, list) => Vec::new(),
            _ => list(name),
        }
    }
}
