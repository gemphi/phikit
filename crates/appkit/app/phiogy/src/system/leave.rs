//! Leave: a caller leaves the call and keeps its line; or its page
//! closes, and the line is gone with it.

use phiecs::prelude::Word;

use crate::board::Board;
use crate::comp::{Hold, Note, Tell};

/// A page leaves its room's call, and every page of the room is told.
/// One that is not in it changes nothing.
pub fn leave(board: &mut Board, line: Word) -> Vec<Note> {
    let joined = board.read(line).is_some_and(|read| read.joined);
    match (joined, board.home(line)) {
        (true, Some(room)) => {
            board.mark(line, |line| line.joined = false);
            board.announce(room)
        }
        _ => Vec::new(),
    }
}

/// A line closes: its page went away. The line ends; had it been in the
/// call, the pages that are left are told. An opening that another has
/// taken the place of closes nothing.
pub fn close(board: &mut Board, hold: Hold) -> Vec<Note> {
    if board.hold(hold.line) != Some(hold) {
        return Vec::new();
    }
    let (joined, room) = (board.read(hold.line).is_some_and(|read| read.joined), board.home(hold.line));
    board.clear(hold.line);
    let mut note = vec![Note { to: hold, tell: Tell::End }];
    // The room is still there when another page has it open.
    let left = room.filter(|_| joined).filter(|room| !board.line_list(*room).is_empty());
    note.extend(left.into_iter().flat_map(|room| board.announce(room)));
    note
}
