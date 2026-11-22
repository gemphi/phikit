//! A note: what the rules have the line tell one page. The rules write
//! notes and send nothing; whoever carries the line delivers them, in
//! the order they were written.

use phiecs::prelude::Word;
use serde_json::{Map, Value};

use super::{Ice, Roster, Wire};

/// One opening of a line. A page that opens its line again takes the
/// place of its old one, and what is still on its way to the old one
/// must not reach the new: the turn tells them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hold {
    pub line: Word,
    pub turn: u64,
}

/// What a page is told.
#[derive(Debug, Clone, PartialEq)]
pub enum Tell {
    /// The first thing on a line: who is in the call, and the servers.
    First(Roster),
    /// Who is in the call now.
    Roster(Roster),
    /// What a caller sent this page.
    Signal { from: String, body: Value },
    /// Nothing, so that a quiet line is not closed.
    Beat,
    /// The line is over.
    End,
}

/// Something to tell one opening of a line.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub to: Hold,
    pub tell: Tell,
}

impl Tell {
    /// What goes on the stream for it; nothing for the end, which is
    /// the stream closing.
    pub fn text(&self, wire: &Wire, ice: &Ice) -> Option<String> {
        let event = |row: Map<String, Value>| Some(format!("{} {}\n\n", wire.event, Value::Object(row)));
        match self {
            Tell::First(roster) => event(Map::from_iter([(wire.roster.clone(), roster.json(wire)), (wire.ice.clone(), ice.json())])),
            Tell::Roster(roster) => event(Map::from_iter([(wire.roster.clone(), roster.json(wire))])),
            Tell::Signal { from, body } => event(Map::from_iter([(wire.from.clone(), Value::String(from.clone())), (wire.signal.clone(), body.clone())])),
            Tell::Beat => Some(format!("{}\n\n", wire.beat)),
            Tell::End => None,
        }
    }
}
