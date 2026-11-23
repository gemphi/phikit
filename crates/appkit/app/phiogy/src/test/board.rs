//! The board as a window reads it, and as a world that stays small.

use super::{board, joining, listed, opened};
use crate::system::{leave, post};

#[test]
fn a_room_is_read_as_who_is_in_its_call_and_who_only_has_it_open() {
    let mut board = board();
    opened(&mut board, "phiogy/lobby", "peer-aaaaaaaa", 1);
    opened(&mut board, "phiogy/lobby", "peer-bbbbbbbb", 2);
    opened(&mut board, "phiogy/lora", "peer-cccccccc", 1);
    post(&mut board, "phiogy/lobby", "guest_2", &joining("peer-bbbbbbbb", true)).unwrap();
    let lobby = board.room("phiogy/lobby").expect("the lobby has a line open");
    assert_eq!(board.roster(lobby), listed(&[("peer-bbbbbbbb", 2)]));
    assert_eq!(board.watch(lobby), listed(&[("peer-aaaaaaaa", 1)]));
    let mut open = board.open();
    open.sort();
    assert_eq!(open, [("phiogy/lobby".to_string(), 1, 2), ("phiogy/lora".to_string(), 0, 1)]);
    assert!(board.room("phiogy/stage").is_none());
}

#[test]
fn a_board_uses_its_words_again_so_it_is_as_large_as_the_most_ever_open() {
    let mut board = board();
    // One page in one room: a room, a line and a caller.
    let hold = opened(&mut board, "phiogy/lobby", "peer-00000000", 0);
    leave::close(&mut board, hold);
    let most = board.made();
    assert_eq!(most, 3);
    // A thousand pages come and go, each with a name of its own in a
    // room of its own: no more words are made.
    for index in 1..1000 {
        let hold = opened(&mut board, &format!("phiogy/room-{index}"), &format!("peer-{index:08}"), index);
        leave::close(&mut board, hold);
    }
    assert_eq!(board.made(), most);
    assert!(board.open().is_empty() && board.every().is_empty());
}

#[test]
fn one_person_with_two_pages_is_one_caller_with_two_lines() {
    let mut board = board();
    let (first, second) = (opened(&mut board, "phiogy/lobby", "peer-aaaaaaaa", 1), opened(&mut board, "phiogy/lora", "peer-bbbbbbbb", 1));
    // Two rooms, two lines, one caller.
    assert_eq!(board.made(), 5);
    assert!(board.mine("phiogy/lobby", "peer-aaaaaaaa", "guest_1").is_ok() && board.mine("phiogy/lora", "peer-bbbbbbbb", "guest_1").is_ok());
    // One page closes: the caller is still there for the other.
    leave::close(&mut board, first);
    assert!(board.owner(second.line).is_some_and(|owner| owner.user == "guest_1"));
    leave::close(&mut board, second);
    assert!(board.every().is_empty());
}
