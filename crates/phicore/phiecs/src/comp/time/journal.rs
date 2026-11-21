//! The journal: every timed aspect over time.
//!
//! Apple keeps no history, only now. A [`Journal`] keeps, for each word and
//! timed aspect, its fields at every tick it changed, and answers with the
//! value at or before any tick. [`MemoryJournal`] is the impl in this crate;
//! the database impl lives outside it (docs/connect.md). Words are named by
//! spelling, so a row means the same in any process.
use super::track::Track;
use crate::comp::stored::Fields;
use std::collections::BTreeMap;
/// An aspect's fields on a word at a tick.
#[derive(Clone, PartialEq, Debug)]
pub struct Row {
    pub tick: u64,
    /// The word's spelling.
    pub word: String,
    /// The aspect's name.
    pub aspect: String,
    pub fields: Fields,
}
pub trait Journal: Send + Sync {
    /// Keeps `row`. A row at a tick already held replaces it.
    fn append(&mut self, row: Row);
    /// The fields of `aspect` on `word` at or before `tick`.
    fn at(&self, word: &str, aspect: &str, tick: u64) -> Option<Fields>;
    /// Every word and aspect's row at or before `tick`: the world as it was.
    fn state_at(&self, tick: u64) -> Vec<Row>;
    /// Every row, by tick.
    fn rows(&self) -> Vec<Row>;
    /// The latest tick held.
    fn last_tick(&self) -> Option<u64>;
    /// Drops every row after `tick`.
    fn truncate_after(&mut self, tick: u64);
}
/// A journal in memory: one [`Track`] a word and aspect.
#[derive(Default)]
pub struct MemoryJournal {
    tracks: BTreeMap<(String, String), Track<Fields>>,
}
impl MemoryJournal {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Journal for MemoryJournal {
    fn append(&mut self, row: Row) {
        self.tracks.entry((row.word, row.aspect)).or_default().push(row.tick, row.fields);
    }
    fn at(&self, word: &str, aspect: &str, tick: u64) -> Option<Fields> {
        self.tracks.get(&(word.to_string(), aspect.to_string()))?.at(tick).cloned()
    }
    fn state_at(&self, tick: u64) -> Vec<Row> {
        let at = |(key, t): (&(String, String), &Track<Fields>)| {
            Some(Row { tick: t.tick_at(tick)?, word: key.0.clone(), aspect: key.1.clone(), fields: t.at(tick)?.clone() })
        };
        self.tracks.iter().filter_map(at).collect()
    }
    fn rows(&self) -> Vec<Row> {
        let mut rows: Vec<Row> = self
            .tracks
            .iter()
            .flat_map(|(k, t)| t.iter().map(|(tick, f)| Row { tick, word: k.0.clone(), aspect: k.1.clone(), fields: f.clone() }))
            .collect();
        rows.sort_by(|a, b| a.tick.cmp(&b.tick));
        rows
    }
    fn last_tick(&self) -> Option<u64> {
        self.tracks.values().filter_map(Track::last_tick).max()
    }
    fn truncate_after(&mut self, tick: u64) {
        self.tracks.values_mut().for_each(|t| t.truncate_after(tick));
        self.tracks.retain(|_, t| !t.is_empty());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::comp::stored::Value;
    fn row(tick: u64, word: &str, n: f64) -> Row {
        Row { tick, word: word.into(), aspect: "pose".into(), fields: vec![("x".into(), Value::Num(n))] }
    }
    #[test]
    fn the_value_at_or_before_a_tick() {
        let mut j = MemoryJournal::new();
        j.append(row(1, "a", 1.0));
        j.append(row(5, "a", 5.0));
        j.append(row(3, "b", 3.0));
        assert_eq!(j.at("a", "pose", 4), Some(row(1, "a", 1.0).fields));
        assert_eq!(j.at("a", "pose", 0), None);
        assert_eq!(j.at("a", "other", 9), None);
        assert_eq!(j.state_at(4).iter().map(|r| (r.word.as_str(), r.tick)).collect::<Vec<_>>(), [("a", 1), ("b", 3)]);
        assert_eq!(j.last_tick(), Some(5));
        assert_eq!(j.rows().iter().map(|r| r.tick).collect::<Vec<_>>(), [1, 3, 5]);
        j.truncate_after(3);
        assert_eq!(j.last_tick(), Some(3));
    }
}
