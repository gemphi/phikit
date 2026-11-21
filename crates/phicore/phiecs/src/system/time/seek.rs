//! Seek: put the world back to how the journal says it was at a tick.
use super::record::Recorder;
use crate::word::Word;
use crate::world::World;
use std::collections::HashSet;
impl World {
    /// Reads every timed aspect back at `tick`: each word gets the value it
    /// had at or before it, and a word that had none yet loses the aspect.
    /// The clock's tick becomes `tick`. Words the world no longer holds are
    /// skipped. Stepping on from here rewrites the journal from there: rows
    /// after the new tick are dropped. Returns how many aspects were set.
    pub fn seek(&mut self, tick: u64) -> usize {
        let Some(mut rec) = self.resources.remove::<Recorder>() else { return 0 };
        let state = rec.journal.state_at(tick);
        if let Some(clock) = self.resources.get_mut::<crate::comp::time::Clock>() {
            clock.tick = tick;
        }
        let mut set = 0;
        for entry in self.stored_entries().into_iter().filter(|e| e.timed && !e.transient) {
            let mut kept: HashSet<Word> = HashSet::new();
            for row in state.iter().filter(|r| r.aspect == entry.name) {
                let Some(word) = self.lexicon.get(&row.word).filter(|w| self.words().contains(w)) else { continue };
                if (entry.write)(self, word, &row.fields) {
                    kept.insert(word);
                    set += 1;
                }
            }
            for word in (entry.holders)(self) {
                if !kept.contains(&word) {
                    (entry.unbind)(self, word);
                }
            }
        }
        rec.last = Some(tick);
        self.resources.insert(rec);
        set
    }
}
