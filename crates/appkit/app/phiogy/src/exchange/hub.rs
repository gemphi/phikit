//! The hub: what every wire of the exchange shares. The board, behind
//! one lock; the way to each open line; and the guests. A rule is run
//! and its notes are handed out while the board is still held, so every
//! page hears things in the order they happened.

use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::guest::Guest;
use super::person::Person;
use super::set::Set;
use crate::board::{Board, lock};
use crate::comp::{Fault, Hold, Note, Post, Roster};
use crate::entity::Caller;
use crate::system::{self, beat, join, leave};

/// What the wires share.
pub struct Hub {
    pub set: Set,
    board: Arc<Mutex<Board>>,
    /// Where what a line is told is put, for the thread that writes it.
    way: Mutex<HashMap<Hold, Sender<String>>>,
    guest: Mutex<Guest>,
    /// What a page asks besides a room's call.
    pub person: Person,
    began: Instant,
    /// How many wires are served now.
    pub live: AtomicUsize,
}

impl Hub {
    pub fn new(set: Set, board: Arc<Mutex<Board>>) -> Hub {
        Hub { set, board, way: Mutex::default(), guest: Mutex::default(), person: Person::own(), began: Instant::now(), live: AtomicUsize::new(0) }
    }

    /// The board's time: milliseconds since the exchange opened.
    pub fn now(&self) -> u64 {
        self.began.elapsed().as_millis() as u64
    }

    /// A number of the board's limits.
    pub fn limit<T>(&self, read: impl FnOnce(&crate::comp::Limit) -> T) -> T {
        read(&lock(&self.board).limit)
    }

    /// Who a page with that cookie is, and a new cookie when it is new.
    pub fn guest(&self, cookie: Option<&str>) -> (Caller, Option<String>) {
        let most = self.limit(|limit| limit.session);
        lock(&self.guest).of(cookie, &self.set, most)
    }

    /// Hand notes out. A line that ends loses its way, which is how the
    /// thread that writes it learns to stop.
    fn deliver(&self, note: Vec<Note>) {
        let mut way = lock(&self.way);
        for note in note {
            match note.tell.text(&self.set.wire, &self.set.ice) {
                Some(text) => way.get(&note.to).into_iter().for_each(|to| _ = to.send(text.clone())),
                None => _ = way.remove(&note.to),
            }
        }
    }

    /// A page opens a room's line: its opening, and where what it is
    /// told will arrive.
    pub fn open(&self, key: &str, peer: &str, caller: &Caller) -> Result<(Hold, Receiver<String>), Fault> {
        let mut board = lock(&self.board);
        let (hold, note) = join::open(&mut board, key, peer, caller, self.now())?;
        let (send, take) = channel();
        lock(&self.way).insert(hold, send);
        self.deliver(note);
        Ok((hold, take))
    }

    /// A page asks something of the line.
    pub fn post(&self, key: &str, user: &str, post: &Post) -> Result<Roster, Fault> {
        let mut board = lock(&self.board);
        let (note, roster) = system::post(&mut board, key, user, post)?;
        self.deliver(note);
        Ok(roster)
    }

    /// A page took what it was told.
    pub fn seen(&self, hold: Hold) {
        beat::seen(&mut lock(&self.board), hold, self.now());
    }

    /// A line closed.
    pub fn close(&self, hold: Hold) {
        let mut board = lock(&self.board);
        let note = leave::close(&mut board, hold);
        lock(&self.way).remove(&hold);
        self.deliver(note);
    }

    /// A beat.
    pub fn beat(&self) {
        let mut board = lock(&self.board);
        let note = beat::beat(&mut board, self.now());
        self.deliver(note);
    }

    /// End every line: the exchange is closing.
    pub fn end(&self) {
        let mut board = lock(&self.board);
        let note: Vec<Note> = board.every().into_iter().flat_map(|hold| leave::close(&mut board, hold)).collect();
        self.deliver(note);
        lock(&self.way).clear();
    }
}
