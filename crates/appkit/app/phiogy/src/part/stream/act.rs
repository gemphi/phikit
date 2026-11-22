//! What a press in the stream does, and a place to press.

use appkit::pui::core::PRect;

/// What a press in the stream does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// Open the picture of that number among the room's.
    Shot(usize),
    /// Open the person of that id beside the room.
    Face(String),
    /// Bring the message of that id into sight.
    Seek(String),
    /// Give the message of that id a mark, or take it back.
    Mark(String, String),
    Reply(String),
    Pin(String),
    Unanswer,
    Send,
    /// Ask for a picture of this disk, to post it.
    Post,
}

/// A place to press, as it was drawn last: where, what it does, and
/// what it is called.
#[derive(Debug, Clone, PartialEq)]
pub struct Spot {
    pub rect: PRect,
    pub act: Act,
    pub word: String,
}
