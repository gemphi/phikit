//! Join: a page opens a room's line, and a page whose line is open joins
//! the room's call.

use phiecs::prelude::Word;

use crate::board::Board;
use crate::comp::{Fault, Hold, Note, PEER, Tell};
use crate::entity::Caller;

/// A page opens a room's line. It is told who is in the call, and it is
/// not in it yet. The same page opening its line again, as it does when
/// the network dropped, takes the place of its old line, which ends;
/// had it been in the call, the room is told it no longer is.
///
/// Refused when the page gave no name a peer may have, when a page of
/// that name is someone else's, and when the room's line is full.
pub fn open(board: &mut Board, key: &str, peer: &str, caller: &Caller, now: u64) -> Result<(Hold, Vec<Note>), Fault> {
    if !board.limit.fits(PEER, peer) {
        return Err(Fault::Peer);
    }
    let room = board.room(key);
    let old = room.and_then(|room| board.line(room, peer));
    let held = room.map_or(0, |room| board.line_list(room).len());
    let theirs = old.map(|line| board.owner(line).is_some_and(|owner| owner.user == caller.user));
    match theirs {
        Some(false) => return Err(Fault::Owner),
        None if held >= board.limit.watch => return Err(Fault::Watch),
        _ => {}
    }
    let before = old.and_then(|line| Some((board.hold(line)?, board.read(line)?.joined)));
    let (room, hold) = board.seat(key, peer, caller, now);
    let mut note = Vec::new();
    if let Some((ended, joined)) = before {
        note.push(Note { to: ended, tell: Tell::End });
        if joined {
            note.extend(board.announce(room));
        }
    }
    note.push(Note { to: hold, tell: Tell::First(board.roster(room)) });
    Ok((hold, note))
}

/// A page joins its room's call, and every page of the room is told.
/// One that is in it already changes nothing. Refused when the call
/// holds as many as it may.
pub fn join(board: &mut Board, line: Word) -> Result<Vec<Note>, Fault> {
    let (Some(room), Some(read)) = (board.home(line), board.read(line)) else { return Err(Fault::Line) };
    if read.joined {
        return Ok(Vec::new());
    }
    if board.roster(room).len() >= board.limit.caller {
        return Err(Fault::Caller);
    }
    board.mark(line, |line| line.joined = true);
    Ok(board.announce(room))
}
