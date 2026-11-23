//! A ring: a room's line held open without being in its call, as a
//! page holds it while the room is shown (`call.ts`), so that a call
//! somebody else began there is seen, with who began it, before this
//! window joins. A window that serves the line itself reads its own
//! board; one that is on another's line listens so.
//!
//! The line is opened on a thread of the ring's own, so a line that is
//! slow to answer, or not there, never holds the window.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use appkit::service::PTable;

use super::{Heard, Line};
use crate::board::lock;
use crate::data;

/// A room's line, listened to.
pub struct Ring {
    /// The room, as the line names it.
    pub key: String,
    /// Who is in its call, as the line said last: what each is called
    /// on the line, and what they call themself.
    heard: Arc<Mutex<Vec<(String, String)>>>,
    stop: Arc<AtomicBool>,
}

impl Ring {
    /// Listen to a room's line at a place, under that name.
    pub fn open(at: &str, key: &str, peer: &str) -> Ring {
        let table = PTable::read(data::PEER);
        let count = |name: &str| Duration::from_millis(table.get(name).parse::<u64>().unwrap_or_default().max(1));
        let (wait, pace) = (count("wait"), count("pace"));
        let (heard, stop) = (Arc::new(Mutex::new(Vec::new())), Arc::new(AtomicBool::new(false)));
        let (kept, flag) = (Arc::clone(&heard), Arc::clone(&stop));
        let (at, room, peer) = (at.to_string(), key.to_string(), peer.to_string());
        _ = std::thread::Builder::new().spawn(move || {
            let Ok(line) = Line::open(&at, &room, &peer, wait) else { return };
            while !flag.load(Ordering::SeqCst) {
                for one in line.take() {
                    match one {
                        Heard::Named(list) => *lock(&kept) = list,
                        Heard::Down | Heard::Refused(_) => return lock(&kept).clear(),
                        Heard::Roster(..) | Heard::Signal(..) => {}
                    }
                }
                std::thread::sleep(pace);
            }
        });
        Ring { key: key.to_string(), heard, stop }
    }

    /// Who is in the room's call now.
    pub fn caller(&self) -> Vec<(String, String)> {
        lock(&self.heard).clone()
    }
}

impl Drop for Ring {
    /// Let go, the line is closed: the thread ends and drops it.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}
