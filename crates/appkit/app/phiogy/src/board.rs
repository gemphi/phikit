//! The board: every room that has a line open, who holds each line and
//! who is in each call. It is a world of words (`phiecs`): a room, a
//! caller and a line are each a word with an aspect bound to it, a line
//! is `on` its room and `of` its caller. The rules (`system/`) are what
//! change it; this file is how they and a window read it.
//!
//! Nothing is kept: a board that is dropped has forgotten every room.
//! A word that is forgotten is used again for the next thing, so the
//! world is as large as the most that was ever open at once, and no
//! larger however long the line runs.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use phiecs::prelude::{Relation, Word, World};

use crate::comp::{Fault, Hold, Limit, Note, Roster, Tell};
use crate::entity::{Caller, Line, Room};

/// What a line is to its room, and to its caller.
pub const ON: &str = "on";
pub const OF: &str = "of";
/// How the board's words are spelled, a number after it.
pub const SLOT: &str = "slot.";

/// Take a lock even after a thread died holding it: the board is whole
/// after every rule.
pub fn lock<T>(held: &Mutex<T>) -> MutexGuard<'_, T> {
    held.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Every room with a line open.
pub struct Board {
    world: World,
    on: Relation,
    of: Relation,
    /// The word of each room, by its key, and of each caller, by who
    /// they are.
    room: HashMap<String, Word>,
    caller: HashMap<String, Word>,
    /// Words nothing is bound to, to use again.
    spare: Vec<Word>,
    made: u64,
    turn: u64,
    pub limit: Limit,
}

impl Board {
    /// A board with nobody on it.
    pub fn new(limit: Limit) -> Board {
        let mut world = World::new();
        let on = world.declare(ON).expect("a new world has room for a relation");
        let of = world.declare(OF).expect("a new world has room for a relation");
        Board { world, on, of, room: HashMap::new(), caller: HashMap::new(), spare: Vec::new(), made: 0, turn: 0, limit }
    }

    /// A word nothing is bound to: one that was forgotten, or a new one.
    fn word(&mut self) -> Word {
        let spelling = match self.spare.pop() {
            Some(word) => self.world.spelling(word).to_string(),
            None => {
                self.made += 1;
                format!("{SLOT}{}", self.made)
            }
        };
        self.world.coin_bare(&spelling)
    }

    fn free(&mut self, word: Word) {
        if self.world.forget(word) {
            self.spare.push(word);
        }
    }

    /// The room of that key, if a line is open in it.
    pub fn room(&self, key: &str) -> Option<Word> {
        self.room.get(key).copied()
    }

    /// The lines of a room, in the order they were opened.
    pub fn line_list(&self, room: Word) -> Vec<Word> {
        self.world.edges.into_of(room).iter().filter(|edge| edge.kind == self.on).map(|edge| edge.from).collect()
    }

    /// The line a page of that name has open in a room.
    pub fn line(&self, room: Word, peer: &str) -> Option<Word> {
        self.line_list(room).into_iter().find(|line| self.read(*line).is_some_and(|line| line.peer == peer))
    }

    pub fn read(&self, line: Word) -> Option<&Line> {
        self.world.read::<Line>(line)
    }

    /// This opening of a line.
    pub fn hold(&self, line: Word) -> Option<Hold> {
        self.read(line).map(|read| Hold { line, turn: read.turn })
    }

    fn of(&self, line: Word) -> Option<Word> {
        self.world.calls_for(line, self.of).first().map(|(word, _)| *word)
    }

    /// Who a line belongs to.
    pub fn owner(&self, line: Word) -> Option<&Caller> {
        self.world.read::<Caller>(self.of(line)?)
    }

    /// The room a line is on.
    pub fn home(&self, line: Word) -> Option<Word> {
        self.world.calls_for(line, self.on).first().map(|(word, _)| *word)
    }

    fn listed(&self, room: Word, joined: bool) -> Roster {
        let one = |line: Word| {
            let read = self.read(line).filter(|read| read.joined == joined)?;
            Some((read.peer.clone(), self.owner(line)?.clone()))
        };
        Roster { list: self.line_list(room).into_iter().filter_map(one).collect() }
    }

    /// Who is in a room's call.
    pub fn roster(&self, room: Word) -> Roster {
        self.listed(room, true)
    }

    /// Who has a room open and is not in its call.
    pub fn watch(&self, room: Word) -> Roster {
        self.listed(room, false)
    }

    /// Tell every page of a room who is in its call.
    pub fn announce(&self, room: Word) -> Vec<Note> {
        let roster = self.roster(room);
        self.line_list(room).into_iter().filter_map(|line| self.hold(line)).map(|to| Note { to, tell: Tell::Roster(roster.clone()) }).collect()
    }

    /// The line a page has open in a room, if whoever asks is the one
    /// who opened it: only they may speak for it.
    pub fn mine(&self, key: &str, peer: &str, user: &str) -> Result<Word, Fault> {
        let line = self.room(key).and_then(|room| self.line(room, peer));
        line.filter(|line| self.owner(*line).is_some_and(|owner| owner.user == user)).ok_or(Fault::Line)
    }

    /// Every line there is.
    pub fn every(&self) -> Vec<Hold> {
        self.world.each::<Line>().map(|(line, read)| Hold { line, turn: read.turn }).collect()
    }

    /// Every room with a line open: its key, how many are in its call
    /// and how many pages have it open.
    pub fn open(&self) -> Vec<(String, usize, usize)> {
        let one = |(word, room): (Word, &Room)| (room.key.clone(), self.roster(word).len(), self.line_list(word).len());
        self.world.each::<Room>().map(one).collect()
    }

    /// How many words the board ever made: the most that were in use
    /// at once.
    pub fn made(&self) -> u64 {
        self.made
    }

    /// Change what is known of a line.
    pub(crate) fn mark(&mut self, line: Word, change: impl FnOnce(&mut Line)) {
        if let Some(line) = self.world.read_mut::<Line>(line) {
            change(line);
        }
    }

    /// Give a page a line in a room, out of the call: a new one, or in
    /// the place of the one it had. The room and the caller are made if
    /// they are not there.
    pub(crate) fn seat(&mut self, key: &str, peer: &str, caller: &Caller, now: u64) -> (Word, Hold) {
        self.turn += 1;
        let fresh = Line { peer: peer.to_string(), joined: false, seen: now, turn: self.turn };
        let room = match self.room(key) {
            Some(room) => room,
            None => {
                let room = self.word();
                self.world.bind(room, Room { key: key.to_string() });
                self.room.insert(key.to_string(), room);
                room
            }
        };
        let line = match self.line(room, peer) {
            Some(line) => line,
            None => {
                let (line, known) = (self.word(), self.caller.get(&caller.user).copied());
                let who = known.unwrap_or_else(|| self.word());
                self.caller.insert(caller.user.clone(), who);
                self.world.bind(who, caller.clone());
                self.world.relate(line, self.on, room);
                self.world.relate(line, self.of, who);
                line
            }
        };
        self.world.bind(line, fresh);
        (room, Hold { line, turn: self.turn })
    }

    /// Forget a line; and its room when no page has it open any more,
    /// and its caller when they have no page left.
    pub(crate) fn clear(&mut self, line: Word) {
        let (room, who) = (self.home(line), self.of(line));
        self.free(line);
        if let Some(room) = room.filter(|room| self.line_list(*room).is_empty()) {
            self.room.retain(|_, word| *word != room);
            self.free(room);
        }
        if let Some(who) = who.filter(|who| self.world.edges.into_of(*who).iter().all(|edge| edge.kind != self.of)) {
            self.caller.retain(|_, word| *word != who);
            self.free(who);
        }
    }
}
