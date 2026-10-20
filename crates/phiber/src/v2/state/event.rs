//! What happened, as a tree of events with key/value pairs — the Outliner's
//! **Status** display mode, and what the status bar points at.
//!
//! The action log (`state::log`) is a line of text a step, on the UI thread
//! alone. A render is a thing with parts — what it was asked for, the
//! settings it was given, how far it has got, how it ended — and most of it
//! happens on the render's own thread. So an event is a node: a title, the
//! pairs that describe it (`Time` → `00:12.34`, `Sample` → `12/256`), and a
//! parent. The store is shared across threads; a revision counter tells a
//! view when to build again.
//!
//! Blender keeps the render's status as `RenderStats` and prints it as one
//! line (`make_renderinfo_string`); this is the same information kept as
//! data, so the line, the Outliner's tree and the journal all read one thing.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

/// One event: a node of the tree.
#[derive(Clone, Debug, PartialEq)]
pub struct BEvent {
    pub id: u64,
    pub parent: Option<u64>,
    pub title: String,
    pub pairs: Vec<(String, String)>,
    /// Seconds since the store was first written, for ordering and display.
    pub at: f64,
}

impl BEvent {
    /// The value of `key`, if the event has it.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }
}

/// How many events are kept; the oldest root and everything under it go first.
pub const CAPACITY: usize = 4000;

static EVENTS: Mutex<Vec<BEvent>> = Mutex::new(Vec::new());
static NEXT: AtomicU64 = AtomicU64::new(1);
static REVISION: AtomicU64 = AtomicU64::new(0);

fn now() -> f64 {
    use std::sync::OnceLock;
    static EPOCH: OnceLock<web_time::Instant> = OnceLock::new();
    EPOCH.get_or_init(web_time::Instant::now).elapsed().as_secs_f64()
}

fn bump() {
    REVISION.fetch_add(1, Ordering::Relaxed);
}

pub struct BEvents;

impl BEvents {
    /// Start an event under `parent` (a root when `None`); its id.
    pub fn open(parent: Option<u64>, title: impl Into<String>, pairs: Vec<(String, String)>) -> u64 {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut all) = EVENTS.lock() {
            all.push(BEvent { id, parent, title: title.into(), pairs, at: now() });
            if all.len() > CAPACITY {
                Self::trim(&mut all);
            }
        }
        bump();
        id
    }

    /// Set `key` on event `id`: replaced where it is, appended where it is not.
    pub fn set(id: u64, key: &str, value: impl Into<String>) {
        let value = value.into();
        if let Ok(mut all) = EVENTS.lock() {
            if let Some(e) = all.iter_mut().find(|e| e.id == id) {
                match e.pairs.iter_mut().find(|(k, _)| k == key) {
                    Some(pair) => pair.1 = value,
                    None => e.pairs.push((key.to_string(), value)),
                }
            }
        }
        bump();
    }

    /// Every event, oldest first.
    pub fn all() -> Vec<BEvent> {
        EVENTS.lock().map(|all| all.clone()).unwrap_or_default()
    }

    /// The newest root event — what the status bar shows.
    pub fn latest_root() -> Option<BEvent> {
        EVENTS.lock().ok()?.iter().rev().find(|e| e.parent.is_none()).cloned()
    }

    /// Changes so far: a view built at one revision is current until this moves.
    pub fn revision() -> u64 {
        REVISION.load(Ordering::Relaxed)
    }

    /// Drop the oldest root and its descendants until under the cap.
    fn trim(all: &mut Vec<BEvent>) {
        while all.len() > CAPACITY {
            let Some(root) = all.iter().find(|e| e.parent.is_none()).map(|e| e.id) else {
                all.remove(0);
                continue;
            };
            let mut gone = vec![root];
            let mut k = 0;
            while k < gone.len() {
                let id = gone[k];
                gone.extend(all.iter().filter(|e| e.parent == Some(id)).map(|e| e.id));
                k += 1;
            }
            all.retain(|e| !gone.contains(&e.id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_event_holds_pairs_and_children_and_moves_the_revision() {
        let before = BEvents::revision();
        let root = BEvents::open(None, "Render Image (test)", vec![("Engine".into(), "Phiray".into())]);
        let child = BEvents::open(Some(root), "Progress", vec![]);
        BEvents::set(child, "Sample", "1/4");
        BEvents::set(child, "Sample", "2/4");
        assert!(BEvents::revision() > before);
        let all = BEvents::all();
        let progress = all.iter().find(|e| e.id == child).unwrap();
        assert_eq!((progress.parent, progress.get("Sample")), (Some(root), Some("2/4")), "a key is replaced, not repeated");
        assert_eq!(all.iter().find(|e| e.id == root).unwrap().get("Engine"), Some("Phiray"));
    }
}
