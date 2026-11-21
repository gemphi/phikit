//! Which aspects a word holds, and when each last changed.
use super::World;
use crate::aspect::Aspect;
use crate::system::event::DidChange;
use crate::word::Word;
impl World {
    /// Counts `A` on `word` changed at the clock's tick, and says so once:
    /// one [`DidChange`] a word, aspect and tick.
    pub(crate) fn mark<A: Aspect>(&mut self, word: Word) {
        let tick = self.clock().tick;
        if self.store_mut::<A>().is_some_and(|s| s.touch(word, tick)) {
            self.emit(DidChange { word, aspect: A::name() });
        }
    }
    /// The tick `A` on `word` was last set or taken `&mut`; `None` if `word`
    /// has no `A`.
    pub fn changed<A: Aspect>(&self, word: Word) -> Option<u64> {
        self.store::<A>()?.changed(word)
    }
    /// Every word whose `A` changed at `tick` or later, with its tick.
    pub fn changed_since<A: Aspect>(&self, tick: u64) -> Vec<(Word, u64)> {
        let Some(store) = self.store::<A>() else { return Vec::new() };
        store.iter().filter_map(|(w, _)| store.changed(w).filter(|t| *t >= tick).map(|t| (w, t))).collect()
    }
    /// The names of the aspects `word` holds, sorted.
    pub fn aspect_list(&self, word: Word) -> Vec<&'static str> {
        let mut names: Vec<&'static str> = self.stores.values().filter(|s| s.has(word)).map(|s| s.name()).collect();
        names.sort_unstable();
        names
    }
}
