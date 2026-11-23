//! The rules of a call: what changes the board. A page opens a room's
//! line and joins its call (`join`), callers pass each other signals
//! (`pass`), a caller leaves or its page closes (`leave`), and a page
//! that went away is dropped (`beat`). Each rule changes the board and
//! writes the notes to deliver; none of them sends anything.

pub mod beat;
pub mod join;
pub mod leave;
pub mod pass;

use crate::board::Board;
use crate::comp::{Fault, Note, Post, Roster};

/// What a page asks of the line, for whoever asks: only the one who
/// opened a page's line may speak for it. They are put in the call or
/// taken out of it, and then what they send is passed on, in the order
/// sent. The answer is who is in the call after it.
pub fn post(board: &mut Board, key: &str, user: &str, post: &Post) -> Result<(Vec<Note>, Roster), Fault> {
    let line = board.mine(key, &post.peer, user)?;
    let mut note = match post.join {
        Some(true) => join::join(board, line)?,
        Some(false) => leave::leave(board, line),
        None => Vec::new(),
    };
    note.extend(pass::pass(board, line, &post.send));
    let roster = board.home(line).map(|room| board.roster(room)).unwrap_or_default();
    Ok((note, roster))
}
