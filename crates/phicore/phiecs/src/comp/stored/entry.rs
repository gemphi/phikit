//! The stored aspects a world knows, by name, as function pointers: the
//! type is gone by the time a row is read back, so each entry carries the
//! code that reads and writes its own type.
use super::{Fields, Stored};
use crate::word::Word;
use crate::world::World;
/// One stored aspect type, erased.
#[derive(Clone, Copy)]
pub struct StoredEntry {
    pub name: &'static str,
    /// Journalled at every tick it changes.
    pub timed: bool,
    /// Never dumped or journalled.
    pub transient: bool,
    /// `word`'s fields, if it holds the aspect.
    pub read: fn(&World, Word) -> Option<Fields>,
    /// Binds the aspect built from the fields; false if they do not fit.
    pub write: fn(&mut World, Word, &Fields) -> bool,
    /// Unbinds the aspect; true if `word` held it.
    pub unbind: fn(&mut World, Word) -> bool,
    /// Every holder whose aspect changed at or after a tick: word, tick, fields.
    pub changed: fn(&World, u64) -> Vec<(Word, u64, Fields)>,
    /// Every word that holds the aspect.
    pub holders: fn(&World) -> Vec<Word>,
}
impl StoredEntry {
    pub fn of<A: Stored>() -> Self {
        Self {
            name: A::KIND,
            timed: A::TIMED,
            transient: A::TRANSIENT,
            read: |w, word| w.read::<A>(word).map(A::fields),
            write: |w, word, f| match A::from_fields(f) {
                Some(a) => {
                    w.bind(word, a);
                    true
                }
                None => false,
            },
            unbind: |w, word| w.unbind::<A>(word).is_some(),
            changed: |w, since| w.changed_since::<A>(since).into_iter().filter_map(|(word, t)| Some((word, t, w.read::<A>(word)?.fields()))).collect(),
            holders: |w| w.each::<A>().map(|(word, _)| word).collect(),
        }
    }
}
/// The registered stored aspects, in the order they were registered; a
/// resource.
#[derive(Default)]
pub struct StoredList {
    entries: Vec<StoredEntry>,
}
impl StoredList {
    pub fn iter(&self) -> impl Iterator<Item = &StoredEntry> {
        self.entries.iter()
    }
    pub fn get(&self, name: &str) -> Option<&StoredEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
    pub(crate) fn add(&mut self, entry: StoredEntry) {
        if self.get(entry.name).is_none() {
            self.entries.push(entry);
        }
    }
}
impl World {
    /// Makes `A` dumpable and, if `A::TIMED`, journalled. Registers its store.
    pub fn register_stored<A: Stored>(&mut self) {
        self.register::<A>();
        if self.resources.get::<StoredList>().is_none() {
            self.resources.insert(StoredList::default());
        }
        if let Some(list) = self.resources.get_mut::<StoredList>() {
            list.add(StoredEntry::of::<A>());
        }
    }
    /// A copy of the registered entries (they are `Copy`).
    pub fn stored_entries(&self) -> Vec<StoredEntry> {
        self.resources.get::<StoredList>().map(|l| l.iter().copied().collect()).unwrap_or_default()
    }
}
