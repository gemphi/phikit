//! A line: one open page in a room, named by the page itself. It hears
//! who is in the room's call whether it is in the call or not.

use phiecs::prelude::Aspect;

/// One open page in a room.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Line {
    /// What the page calls itself.
    pub peer: String,
    /// Whether it is in the call.
    pub joined: bool,
    /// When it last took what it was told, in the board's milliseconds.
    pub seen: u64,
    /// Which opening of the line this is.
    pub turn: u64,
}

impl Aspect for Line {}
