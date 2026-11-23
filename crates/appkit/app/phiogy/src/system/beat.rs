//! Beat: a line that says nothing is told a beat now and then, so that
//! nothing between the page and the line closes a quiet stream; and a
//! page that took nothing for too long went away, and is dropped as if
//! it had closed. How often and how long are `data/limit.tsv`.

use crate::board::Board;
use crate::comp::{Hold, Note, Tell};
use crate::system::leave;

/// A page took what it was told: it is there.
pub fn seen(board: &mut Board, hold: Hold, now: u64) {
    if board.hold(hold.line) == Some(hold) {
        board.mark(hold.line, |line| line.seen = now);
    }
}

/// A beat. Every page that took nothing for longer than the limit is
/// dropped, and the pages left in its room are told if it was in the
/// call; every other page is told the beat.
pub fn beat(board: &mut Board, now: u64) -> Vec<Note> {
    let limit = board.limit.gone;
    let late = |hold: &Hold| board.read(hold.line).is_some_and(|read| now.saturating_sub(read.seen) > limit);
    let gone: Vec<Hold> = board.every().into_iter().filter(late).collect();
    let mut note: Vec<Note> = gone.into_iter().flat_map(|hold| leave::close(board, hold)).collect();
    note.extend(board.every().into_iter().map(|to| Note { to, tell: Tell::Beat }));
    note
}
