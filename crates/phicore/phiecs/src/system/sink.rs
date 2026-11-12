//! Sinks: the one way out for rows and events.
//!
//! The Readout stage hands every journal row and every lifecycle event to
//! each attached [`Sink`]. This crate has the memory journal and a
//! [`MemorySink`]; the session file (philog) and the database (phiora) are
//! sinks supplied from outside (docs/connect.md).
use crate::comp::time::Row;
use std::sync::{Arc, Mutex};
pub trait Sink: Send + Sync {
    /// A timed aspect changed: its fields at the row's tick.
    fn row(&mut self, row: &Row);
    /// A lifecycle event at `tick`: its name (`did_add`, `will_remove`) and
    /// the word's spelling.
    fn event(&mut self, _tick: u64, _name: &str, _word: &str) {}
}
/// A sink that keeps what it is given. Clones share what they keep, so one
/// handle can be attached to the world while another is read.
#[derive(Clone, Default)]
pub struct MemorySink {
    rows: Arc<Mutex<Vec<Row>>>,
    events: Arc<Mutex<Vec<(u64, String, String)>>>,
}
impl MemorySink {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn rows(&self) -> Vec<Row> {
        self.rows.lock().map(|r| r.clone()).unwrap_or_default()
    }
    /// `(tick, name, word)` in the order they came.
    pub fn events(&self) -> Vec<(u64, String, String)> {
        self.events.lock().map(|e| e.clone()).unwrap_or_default()
    }
}
impl Sink for MemorySink {
    fn row(&mut self, row: &Row) {
        if let Ok(mut r) = self.rows.lock() {
            r.push(row.clone());
        }
    }
    fn event(&mut self, tick: u64, name: &str, word: &str) {
        if let Ok(mut e) = self.events.lock() {
            e.push((tick, name.to_string(), word.to_string()));
        }
    }
}
