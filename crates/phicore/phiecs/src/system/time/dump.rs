//! Dump and load: a whole world as rows, and back.
//!
//! One row a word and stored, non-transient aspect, at the clock's tick,
//! plus a `kind` row (`name`) and a `parent` row (`word`) where the word has
//! them. They are the same rows the journal keeps, so one reader serves both.
use crate::comp::stored::{Fields, Value, field};
use crate::comp::time::Row;
use crate::world::World;
fn text(key: &str, v: &str) -> Fields {
    vec![(key.to_string(), Value::Text(v.to_string()))]
}
impl World {
    /// The world as rows, words in the order they were coined.
    pub fn dump(&self) -> Vec<Row> {
        let tick = self.clock().tick;
        let entries: Vec<_> = self.stored_entries().into_iter().filter(|e| !e.transient).collect();
        let mut rows = Vec::new();
        for &word in self.words() {
            let name = self.spelling(word).to_string();
            let mut row = |aspect: &str, fields: Fields| rows.push(Row { tick, word: name.clone(), aspect: aspect.to_string(), fields });
            if let Some(kind) = self.kind_of(word).and_then(|k| self.kind_name(k)) {
                row("kind", text("name", kind));
            }
            for e in &entries {
                if let Some(fields) = (e.read)(self, word) {
                    row(e.name, fields);
                }
            }
            if let Some(parent) = self.tree.parent(word) {
                row("parent", text("word", self.spelling(parent)));
            }
        }
        rows
    }
    /// Adds the words and aspects in `rows`; the clock takes the latest tick.
    /// A row for an unknown kind or aspect, or with fields that do not fit,
    /// is skipped. Returns how many rows were applied.
    pub fn load(&mut self, rows: &[Row]) -> usize {
        let entries = self.stored_entries();
        let mut applied = 0;
        for row in rows {
            let word = self.coin_bare(&row.word);
            let ok = match row.aspect.as_str() {
                "kind" => match field(&row.fields, "name").and_then(Value::text).and_then(|n| self.kind(n)) {
                    Some(k) => {
                        self.set_kind(word, k);
                        true
                    }
                    None => false,
                },
                "parent" => true, // after every word is known
                other => entries.iter().find(|e| e.name == other).is_some_and(|e| (e.write)(self, word, &row.fields)),
            };
            applied += usize::from(ok);
        }
        for row in rows.iter().filter(|r| r.aspect == "parent") {
            let Some(parent) = field(&row.fields, "word").and_then(Value::text) else { continue };
            let (child, parent) = (self.coin_bare(&row.word), self.coin_bare(parent));
            self.set_parent(child, parent, false);
        }
        let latest = rows.iter().map(|r| r.tick).max().unwrap_or(0);
        if let Some(clock) = self.resources.get_mut::<crate::comp::time::Clock>() {
            clock.tick = clock.tick.max(latest);
        }
        applied
    }
}
