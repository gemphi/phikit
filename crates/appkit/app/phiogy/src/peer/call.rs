//! A call this window is in: it holds a room's line as a caller, joins
//! the room's call, and keeps a connection to each other caller. It
//! runs on a thread of its own, so what the line and the connections
//! bring is taken in whether the window is drawing or not; the window
//! asks it things and reads what it has seen.
//!
//! Dropping it leaves the call: the line closes, and the room is told.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use appkit::service::PTable;

use super::engine::{Engine, Maker, Own};
use super::line::{Heard, Line};
use super::mesh::{Far, Mesh};
use crate::board::lock;
use crate::data;

/// What the window asks of a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ask {
    Hush(bool),
    Camera(bool),
    Share(bool),
    Deaf(bool),
    End,
}

/// What a call has seen, for the window to read.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Seen {
    /// Whether the line has this caller in the room's call.
    pub joined: bool,
    /// Whether this caller is muted.
    pub quiet: bool,
    /// The other callers.
    pub far: Vec<Far>,
    /// What everyone in the call calls themself, by their name on the
    /// line, as the line said last.
    pub named: Vec<(String, String)>,
    /// Why the call is over, when it is: the line refused with that
    /// status, or went away.
    pub over: Option<String>,
    /// What carries the call.
    pub engine: String,
    /// This caller's own side: its camera, its screen, how loud it is
    /// and what the system did not give it.
    pub own: Own,
}

/// A call this window is in.
pub struct Call {
    /// What this window calls itself on the line.
    pub peer: String,
    /// The room, as the line names it.
    pub key: String,
    /// When it was begun.
    pub since: std::time::Instant,
    ask: Sender<Ask>,
    seen: Arc<Mutex<Seen>>,
}

/// A name for this window on a line: how `data/peer.tsv` says it
/// begins, and digits chosen by chance. Nothing where there is no
/// chance to be had, in a browser.
pub fn name() -> Option<String> {
    #[cfg(not(target_family = "wasm"))]
    return Some(format!("{}{:016x}", PTable::read(data::PEER).get("name"), rand::random::<u64>()));
    #[cfg(target_family = "wasm")]
    None
}

impl Call {
    /// Open a room's line at a place as a caller of that name, and join
    /// the room's call. The engine is made on the call's own thread.
    /// `Err` when the line is not there or refuses the name.
    pub fn start(at: &str, key: &str, peer: &str, make: Maker) -> std::io::Result<Call> {
        let table = PTable::read(data::PEER);
        let count = |name: &str| table.get(name).parse::<u64>().unwrap_or_default();
        let line = Line::open(at, key, peer, Duration::from_millis(count("wait")))?;
        line.join(true);
        let (ask, asked) = channel();
        let seen = Arc::new(Mutex::new(Seen::default()));
        let (shown, pace) = (Arc::clone(&seen), Duration::from_millis(count("pace").max(1)));
        std::thread::Builder::new().spawn(move || run(line, make(), &asked, &shown, pace))?;
        Ok(Call { peer: peer.to_string(), key: key.to_string(), since: std::time::Instant::now(), ask, seen })
    }

    /// What the call has seen so far.
    pub fn seen(&self) -> Seen {
        lock(&self.seen).clone()
    }

    /// What the call sees, kept current, for a thread that is not the
    /// window's to read.
    pub fn watch(&self) -> Arc<Mutex<Seen>> {
        Arc::clone(&self.seen)
    }

    /// Mute, or unmute.
    pub fn hush(&self, quiet: bool) {
        _ = self.ask.send(Ask::Hush(quiet));
    }

    /// Turn the camera on, or off.
    pub fn camera(&self, on: bool) {
        _ = self.ask.send(Ask::Camera(on));
    }

    /// Share a screen, or stop.
    pub fn share(&self, on: bool) {
        _ = self.ask.send(Ask::Share(on));
    }

    /// Hear nobody, or hear them again.
    pub fn deaf(&self, on: bool) {
        _ = self.ask.send(Ask::Deaf(on));
    }
}

impl Drop for Call {
    fn drop(&mut self) {
        _ = self.ask.send(Ask::End);
    }
}

/// The call's own thread: take what was asked, what the line brought
/// and what the connections made, and send what follows from it, until
/// the call is ended or the line is gone.
fn run(line: Line, mut engine: Box<dyn Engine>, asked: &Receiver<Ask>, seen: &Mutex<Seen>, pace: Duration) {
    let mut mesh = Mesh::new(&line.peer);
    let (mut joined, mut over, mut named) = (false, None, Vec::new());
    lock(seen).engine = engine.name().to_string();
    'call: loop {
        let mut out = Vec::new();
        for ask in asked.try_iter() {
            match ask {
                Ask::Hush(quiet) => out.extend(mesh.hush(quiet, engine.as_mut())),
                Ask::Camera(on) => engine.camera(on),
                Ask::Share(on) => engine.share(on),
                Ask::Deaf(on) => engine.deaf(on),
                Ask::End => break 'call,
            }
        }
        for heard in line.take() {
            match heard {
                Heard::Roster(roster, ice) => {
                    mesh.ice = ice.unwrap_or_else(|| std::mem::take(&mut mesh.ice));
                    joined = roster.contains(&line.peer);
                    out.extend(mesh.sync(&roster, engine.as_mut()));
                }
                Heard::Named(list) => named = list,
                Heard::Signal(from, body) => mesh.hear(&from, &body),
                Heard::Refused(status) => over = Some(status),
                Heard::Down => over = Some(String::new()),
            }
        }
        out.extend(mesh.turn());
        out.into_iter().for_each(|(to, signal)| line.send(&to, signal));
        *lock(seen) = Seen { joined: joined && over.is_none(), quiet: mesh.quiet(), far: mesh.far(), named: named.clone(), over: over.clone(), engine: engine.name().to_string(), own: engine.own() };
        if over.is_some() {
            break;
        }
        std::thread::sleep(pace);
    }
    mesh.close();
    lock(seen).joined = false;
}
