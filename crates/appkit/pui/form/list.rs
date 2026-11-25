//! Lists and state from data, for any form: a table of lists says, for
//! each list, the table its rows come from, which columns give a choice's
//! value, words, icon and colour, which rows are kept, and when the list
//! is shown at all. Its values are kept by path in whatever states the
//! app gives, each under the first step of the paths it keeps. Nothing
//! of a list is written in code.
//!
//! A row of the list table: the list's name, its table, the value's
//! column, the words' column (`text:` before it, the words are the app's
//! text for that column's value), the icon's column, the colour's
//! columns (three, red green blue, `/255` after them where they run to
//! 255), the rows it keeps (`column=path`: those whose column holds
//! what the path keeps), whether it keeps each value once (`once`), when
//! it is shown (`path=value`), and, where a list has them, the columns
//! of its picture (a grid's tile), its line, the note at its end, its
//! depth in a tree, its face (`sheet columns cell`), whether a tree's
//! line is open (`1`) or can open (`0`), its level (0 to 1, a bar at a
//! row's end: how loud a caller is), and what can be done with it where
//! it stands (`word:icon`, `:1` after one that is on, between spaces).
//! `-` for none, and a row may stop before the last three. A path may
//! hold `{path}`: what that path keeps, put in its place.

use std::rc::Rc;

use super::{PChoice, PDo, PSource};
use crate::pui::core::PColor;
use crate::pui::state::{PState, step};
use crate::service::PTable;

/// Where a form's lists find their tables' rows, by the table's name.
pub trait PRows {
    fn rows(&self, table: &str) -> Vec<Vec<String>>;
}

/// Rows found by a function: an app's tables, handed over without a type
/// of its own.
impl<F: Fn(&str) -> Vec<Vec<String>>> PRows for F {
    fn rows(&self, table: &str) -> Vec<Vec<String>> {
        self(table)
    }
}

/// The lists and states of a form, from data.
pub struct PListed {
    list: Vec<Vec<String>>,
    rows: Rc<dyn PRows>,
    text: Rc<PTable>,
    state: Vec<(String, Rc<dyn PState>)>,
}

impl PListed {
    /// The lists a table names, their rows from `rows`, their words from
    /// the app's text, their values kept by path in the states given,
    /// each under the first step of its paths.
    pub fn new(table: &str, rows: Rc<dyn PRows>, text: Rc<PTable>, state: Vec<(String, Rc<dyn PState>)>) -> PListed {
        let list = table.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|l| l.split('\t').map(str::to_string).collect()).collect();
        PListed { list, rows, text, state }
    }

    /// A path with every `{path}` in it put in as what that path keeps.
    fn filled(&self, path: &str) -> String {
        let mut out = path.to_string();
        while let (Some(open), Some(close)) = (out.find('{'), out.find('}')) {
            let inner = self.get(&out[open + 1..close]);
            out.replace_range(open..=close, &inner);
        }
        out
    }

    fn keeper(&self, path: &str) -> Option<&Rc<dyn PState>> {
        let (first, _) = step(path);
        self.state.iter().find(|(name, _)| name == first).map(|(_, state)| state)
    }
}

impl PState for PListed {
    fn get(&self, path: &str) -> String {
        let path = self.filled(path);
        self.keeper(&path).map(|state| state.get(&path)).unwrap_or_default()
    }

    fn set(&self, path: &str, value: &str) {
        let path = self.filled(path);
        self.keeper(&path).into_iter().for_each(|state| state.set(&path, value));
    }
}

impl PSource for PListed {
    fn list(&self, name: &str) -> Vec<PChoice> {
        let Some(spec) = self.list.iter().find(|row| row.first().is_some_and(|n| n == name)) else { return Vec::new() };
        let at = |i: usize| spec.get(i).map(String::as_str).filter(|c| *c != "-").unwrap_or("");
        let shown = match at(8).split_once('=') {
            Some((path, value)) => self.get(path) == value,
            None => true,
        };
        let kept = at(6).split_once('=').map(|(column, path)| (column.parse::<usize>().unwrap_or_default(), self.get(path)));
        let column = |row: &Vec<String>, spec: &str| spec.parse::<usize>().ok().and_then(|i| row.get(i).cloned()).unwrap_or_default();
        let words = |row: &Vec<String>| match at(3).strip_prefix("text:") {
            Some(spec) => self.text.get(&column(row, spec)).to_string(),
            None => column(row, at(3)),
        };
        let (colour, range) = at(5).split_once('/').map_or((at(5), 1.0), |(c, r)| (c, r.parse().unwrap_or(1.0)));
        let paint = |row: &Vec<String>| {
            let v: Vec<f32> = colour.split(',').filter_map(|c| c.parse::<usize>().ok()).filter_map(|i| row.get(i)?.parse().ok()).map(|v: f32| v / range).collect();
            (v.len() == 3).then(|| PColor::new(v[0], v[1], v[2], 1.0))
        };
        let mut out: Vec<PChoice> = Vec::new();
        for row in self.rows.rows(at(1)).iter().filter(|row| kept.as_ref().is_none_or(|(c, v)| row.get(*c) == Some(v))).filter(|_| shown) {
            let choice = PChoice { value: column(row, at(2)), label: words(row), icon: column(row, at(4)), colour: paint(row), picture: column(row, at(9)), note: column(row, at(10)), end: column(row, at(11)), depth: column(row, at(12)).parse().unwrap_or_default(), face: face(&column(row, at(13))), fold: fold(&column(row, at(14))), level: column(row, at(15)).parse().ok(), deed: deed(&column(row, at(16))) };
            match at(7) == "once" && out.iter().any(|c| c.value == choice.value) {
                true => {}
                false => out.push(choice),
            }
        }
        out
    }
}

/// What can be done with a choice, as a column writes it: between
/// spaces, each `word:icon`, `:1` after one that is on.
fn deed(cell: &str) -> Vec<PDo> {
    cell.split_whitespace().filter_map(|one| {
        let mut part = one.split(':');
        let (word, icon) = (part.next()?, part.next()?);
        Some(PDo { word: word.to_string(), icon: icon.to_string(), on: part.next() == Some("1") })
    }).collect()
}

/// Whether a tree's line is open (`1`), can open (`0`), or neither.
fn fold(cell: &str) -> Option<bool> {
    match cell {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

/// A face as a column writes it: a sheet, how many columns it has, and
/// the cell, between spaces; nothing for anything else.
fn face(cell: &str) -> Option<(String, u32, u32)> {
    let mut part = cell.split_whitespace();
    let (sheet, column, index) = (part.next()?, part.next()?.parse().ok()?, part.next()?.parse().ok()?);
    Some((sheet.to_string(), column, index))
}
