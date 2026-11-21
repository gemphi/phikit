//! Record: the Readout system that writes the journal.
//!
//! Each step, for every timed, non-transient stored aspect that changed
//! since the last record, one [`Row`] goes to the journal and to every
//! attached [`Sink`]; so do the step's [`DidAdd`] and [`WillRemove`]. A
//! change made between steps (stamped with the tick the last step ended on)
//! is caught by the next record. A row equal to the journal's is not
//! written twice. An aspect taken away is not recorded; the journal keeps
//! its last value.
use crate::comp::stored::Stored;
use crate::comp::time::{Journal, MemoryJournal, Row};
use crate::system::event::{DidAdd, EventQueue, WillRemove};
use crate::system::sink::Sink;
use crate::system::{FnSystem, Stage};
use crate::word::Word;
use crate::world::World;
/// Where history goes, a resource: the journal and the sinks.
pub struct Recorder {
    pub journal: Box<dyn Journal>,
    pub(crate) sinks: Vec<Box<dyn Sink>>,
    /// The tick the last record ran at.
    pub(crate) last: Option<u64>,
}
impl Default for Recorder {
    fn default() -> Self {
        Self { journal: Box::new(MemoryJournal::new()), sinks: Vec::new(), last: None }
    }
}
/// The record system, at [`Stage::Readout`].
pub fn record() -> FnSystem<impl FnMut(&mut World, f32) + Send + Sync> {
    FnSystem::new("record", |world, _| world.record_now()).at(Stage::Readout)
}
impl World {
    /// Attaches `sink`: it gets every row and event from now on.
    pub fn attach(&mut self, sink: impl Sink + 'static) {
        if let Some(r) = self.resources.get_mut::<Recorder>() {
            r.sinks.push(Box::new(sink));
        }
    }
    /// Swaps the journal; the old one is handed back.
    pub fn set_journal(&mut self, journal: Box<dyn Journal>) -> Option<Box<dyn Journal>> {
        self.resources.get_mut::<Recorder>().map(|r| std::mem::replace(&mut r.journal, journal))
    }
    /// The value of `A` on `word` at or before `tick`, from the journal.
    pub fn at<A: Stored>(&self, word: Word, tick: u64) -> Option<A> {
        let fields = self.resources.get::<Recorder>()?.journal.at(self.spelling(word), A::KIND, tick)?;
        A::from_fields(&fields)
    }
    /// Every journal row, by tick.
    pub fn journal_rows(&self) -> Vec<Row> {
        self.resources.get::<Recorder>().map(|r| r.journal.rows()).unwrap_or_default()
    }
    /// Records this tick now. [`record`] runs this at Readout.
    pub fn record_now(&mut self) {
        let Some(mut rec) = self.resources.remove::<Recorder>() else { return };
        let tick = self.clock().tick;
        if rec.journal.last_tick().is_some_and(|t| t > tick) {
            rec.journal.truncate_after(tick); // resumed in the past: that future is gone
        }
        let since = rec.last.unwrap_or(0);
        for entry in self.stored_entries().into_iter().filter(|e| e.timed && !e.transient) {
            for (word, changed, fields) in (entry.changed)(self, since) {
                let name = self.spelling(word).to_string();
                if rec.journal.at(&name, entry.name, changed).as_ref() == Some(&fields) {
                    continue;
                }
                let row = Row { tick: changed, word: name, aspect: entry.name.to_string(), fields };
                rec.sinks.iter_mut().for_each(|s| s.row(&row));
                rec.journal.append(row);
            }
        }
        if let Some(q) = self.resources.get::<EventQueue>() {
            let named = |w: Word| self.spelling(w).to_string();
            for e in q.read_now::<DidAdd>() {
                rec.sinks.iter_mut().for_each(|s| s.event(tick, "did_add", &named(e.0)));
            }
            for e in q.read_now::<WillRemove>() {
                rec.sinks.iter_mut().for_each(|s| s.event(tick, "will_remove", &named(e.0)));
            }
        }
        rec.last = Some(tick);
        self.resources.insert(rec);
    }
}
