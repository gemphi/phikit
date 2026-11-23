//! What a hand does: a pointer pressed, let go, moved or scrolled, a key
//! pressed or let go. gpui's input is transformed into this file later and
//! brings the buttons, the modifiers and the gestures with it.

use super::PPoint;

/// One thing a hand did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PEvent {
    PointerDown(PPoint),
    PointerUp(PPoint),
    PointerMove(PPoint),
    /// The pointer's place, and how far the wheel turned.
    Scroll(PPoint, PPoint),
    KeyDown(char),
    KeyUp(char),
}

impl PEvent {
    /// The characters the keys that write nothing are sent as. Backspace,
    /// Enter and Escape have characters of their own. The arrows, Delete,
    /// Home and End are sent as Apple's AppKit sends them: characters of
    /// the range it keeps for function keys (`NSUpArrowFunctionKey` and
    /// on, 0xF700 to 0xF8FF).
    pub const BACKSPACE: char = '\u{8}';
    pub const ENTER: char = '\n';
    pub const ESCAPE: char = '\u{1b}';
    pub const UP: char = '\u{F700}';
    pub const DOWN: char = '\u{F701}';
    pub const LEFT: char = '\u{F702}';
    pub const RIGHT: char = '\u{F703}';
    pub const DELETE: char = '\u{F728}';
    pub const HOME: char = '\u{F729}';
    pub const END: char = '\u{F72B}';

    /// Whether a character is one of the range kept for keys that write
    /// nothing.
    pub fn silent(key: char) -> bool {
        ('\u{F700}'..='\u{F8FF}').contains(&key)
    }

    /// Where the pointer was, for an event that has a place. A key has
    /// none: it goes to whoever is listening, not to what is under it.
    pub fn at(&self) -> Option<PPoint> {
        match *self {
            PEvent::PointerDown(at) | PEvent::PointerUp(at) | PEvent::PointerMove(at) | PEvent::Scroll(at, _) => Some(at),
            PEvent::KeyDown(_) | PEvent::KeyUp(_) => None,
        }
    }

    /// The same event as a view at `origin` sees it: a place in the
    /// parent's coordinates becomes one in the view's own.
    pub fn seen_from(&self, origin: PPoint) -> PEvent {
        match *self {
            PEvent::PointerDown(at) => PEvent::PointerDown(at.minus(origin)),
            PEvent::PointerUp(at) => PEvent::PointerUp(at.minus(origin)),
            PEvent::PointerMove(at) => PEvent::PointerMove(at.minus(origin)),
            PEvent::Scroll(at, by) => PEvent::Scroll(at.minus(origin), by),
            PEvent::KeyDown(_) | PEvent::KeyUp(_) => *self,
        }
    }
}
