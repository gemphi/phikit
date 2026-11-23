//! Pass: what one caller sends another goes through the line, which
//! does not read it. It goes only between two who are both in the call,
//! and in the order it was sent, so an offer never arrives after its
//! candidates.

use phiecs::prelude::Word;

use crate::board::Board;
use crate::comp::{Note, Signal, Tell};

/// Pass a page's signals on to the pages they are for. One for a page
/// that is not in the call, or from one that is not, goes nowhere.
pub fn pass(board: &Board, line: Word, send: &[Signal]) -> Vec<Note> {
    let (Some(room), Some(me)) = (board.home(line), board.read(line).filter(|read| read.joined)) else { return Vec::new() };
    let one = |signal: &Signal| {
        let to = board.line(room, &signal.to).filter(|to| board.read(*to).is_some_and(|read| read.joined))?;
        Some(Note { to: board.hold(to)?, tell: Tell::Signal { from: me.peer.clone(), body: signal.body.clone() } })
    };
    send.iter().filter_map(one).collect()
}
