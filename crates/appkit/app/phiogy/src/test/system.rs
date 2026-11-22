//! The rules of a call, with no network: what the Node line's own tests
//! cover (`puikit/app/phiora-server/test/call.test.mjs`), and the beat.

use serde_json::json;

use super::{board, bound, guest, joining, listed, opened, roster, sending};
use crate::comp::{Fault, Limit, Note, Post, Tell};
use crate::system::{beat, join, leave, post};

const LOBBY: &str = "demo/lobby";
const A: &str = "peer-aaaaaaaa";
const B: &str = "peer-bbbbbbbb";

#[test]
fn two_pages_in_a_room_find_each_other_and_trade_signals() {
    let mut board = board();
    // Each page's line begins by saying who is in the call: nobody.
    let (a, first) = join::open(&mut board, LOBBY, A, &guest(1), 0).unwrap();
    assert_eq!(first, [Note { to: a, tell: Tell::First(listed(&[])) }]);
    let b = opened(&mut board, LOBBY, B, 2);
    // A starts a call: B, who has only the room open, hears of it.
    let (note, answer) = post(&mut board, LOBBY, "guest_1", &joining(A, true)).unwrap();
    assert_eq!(note, [roster(a, &[(A, 1)]), roster(b, &[(A, 1)])]);
    assert_eq!(answer, listed(&[(A, 1)]));
    // A signal to someone who has not joined goes nowhere.
    let (note, _) = post(&mut board, LOBBY, "guest_1", &sending(A, B, &[json!({ "lost": true })])).unwrap();
    assert!(note.is_empty(), "{note:?}");
    // Once both are in, it arrives, in the order sent.
    let (note, _) = post(&mut board, LOBBY, "guest_2", &joining(B, true)).unwrap();
    assert_eq!(note, [roster(a, &[(A, 1), (B, 2)]), roster(b, &[(A, 1), (B, 2)])]);
    let (offer, candidate) = (json!({ "description": { "type": "offer", "sdp": "v=0" } }), json!({ "candidate": { "candidate": "c1" } }));
    let (note, _) = post(&mut board, LOBBY, "guest_1", &sending(A, B, &[offer.clone(), candidate.clone()])).unwrap();
    let from = |body| Note { to: b, tell: Tell::Signal { from: A.to_string(), body } };
    assert_eq!(note, [from(offer), from(candidate)]);
    // B's page goes away: A's roster no longer holds them.
    assert_eq!(leave::close(&mut board, b), [Note { to: b, tell: Tell::End }, roster(a, &[(A, 1)])]);
    let (note, _) = post(&mut board, LOBBY, "guest_1", &joining(A, false)).unwrap();
    assert_eq!(note, [roster(a, &[])]);
    // The last page closes, and the room is forgotten.
    assert_eq!(leave::close(&mut board, a), [Note { to: a, tell: Tell::End }]);
    assert!(board.open().is_empty() && board.room(LOBBY).is_none());
}

#[test]
fn a_peer_belongs_to_whoever_opened_its_line() {
    let (mut board, den) = (board(), "demo/den");
    opened(&mut board, den, "peer-cccccccc", 1);
    opened(&mut board, den, "peer-dddddddd", 2);
    // Nobody speaks for another's page, or for a page with no line.
    assert_eq!(post(&mut board, den, "guest_2", &joining("peer-cccccccc", true)).unwrap_err(), Fault::Line);
    assert_eq!(post(&mut board, den, "guest_2", &joining("peer-eeeeeeee", true)).unwrap_err(), Fault::Line);
    // A page must name itself, and nobody takes another's name.
    assert_eq!(join::open(&mut board, den, "short", &guest(3), 0).unwrap_err(), Fault::Peer);
    assert_eq!(join::open(&mut board, den, "peer-cccccccc", &guest(2), 0).unwrap_err(), Fault::Owner);
    assert_eq!(board.open(), [(den.to_string(), 0, 2)]);
}

#[test]
fn a_call_holds_as_many_as_the_limit_says_which_is_eight() {
    let (mut board, hall) = (board(), "demo/hall");
    let most = Limit::own().caller;
    assert_eq!(most, 8);
    let peer: Vec<String> = (0..=most).map(|index| format!("peer-full-{index}xxx")).collect();
    peer.iter().enumerate().for_each(|(index, peer)| _ = opened(&mut board, hall, peer, index));
    let got: Vec<bool> = peer.iter().enumerate().map(|(index, peer)| post(&mut board, hall, &format!("guest_{index}"), &joining(peer, true)).is_ok()).collect();
    assert_eq!(got, [vec![true; most], vec![false]].concat());
    let last = post(&mut board, hall, &format!("guest_{most}"), &joining(&peer[most], true));
    assert_eq!(last.unwrap_err(), Fault::Caller);
    // One who is in it may ask again; one leaves, and the ninth is in.
    assert!(post(&mut board, hall, "guest_0", &joining(&peer[0], true)).is_ok_and(|(note, _)| note.is_empty()));
    assert!(post(&mut board, hall, "guest_0", &joining(&peer[0], false)).is_ok());
    assert!(post(&mut board, hall, &format!("guest_{most}"), &joining(&peer[most], true)).is_ok());
}

#[test]
fn a_room_s_line_holds_as_many_pages_as_the_limit_says() {
    assert_eq!(Limit::own().watch, 256);
    let mut board = bound("watch\t2");
    opened(&mut board, LOBBY, A, 1);
    opened(&mut board, LOBBY, B, 2);
    assert_eq!(join::open(&mut board, LOBBY, "peer-cccccccc", &guest(3), 0).unwrap_err(), Fault::Watch);
    // A page that has a line opens it again whether the room is full or not.
    assert!(join::open(&mut board, LOBBY, A, &guest(1), 0).is_ok());
    // Another room has its own count.
    assert!(join::open(&mut board, "demo/den", "peer-cccccccc", &guest(3), 0).is_ok());
}

#[test]
fn the_beat_drops_a_page_that_went_away() {
    let mut board = board();
    let gone = Limit::own().gone;
    let (a, b) = (opened(&mut board, LOBBY, A, 1), opened(&mut board, LOBBY, B, 2));
    post(&mut board, LOBBY, "guest_1", &joining(A, true)).unwrap();
    post(&mut board, LOBBY, "guest_2", &joining(B, true)).unwrap();
    // While both take what they are told, a beat is all either hears.
    beat::seen(&mut board, a, gone);
    beat::seen(&mut board, b, gone);
    let told = |to| Note { to, tell: Tell::Beat };
    assert_eq!(beat::beat(&mut board, gone + 1), [told(a), told(b)]);
    // B takes nothing for longer than the limit: it went away. Its line
    // ends, A is told who is left, and only A is told the beat.
    beat::seen(&mut board, a, gone * 2);
    assert_eq!(beat::beat(&mut board, gone * 2 + 1), [Note { to: b, tell: Tell::End }, roster(a, &[(A, 1)]), told(a)]);
    assert_eq!(board.open(), [(LOBBY.to_string(), 1, 1)]);
    // What is on its way to the page that went changes nothing.
    beat::seen(&mut board, b, gone * 2 + 2);
    assert!(leave::close(&mut board, b).is_empty());
    assert_eq!(post(&mut board, LOBBY, "guest_2", &joining(B, true)).unwrap_err(), Fault::Line);
}

#[test]
fn a_page_that_opens_its_line_again_takes_the_place_of_its_old_one() {
    let mut board = board();
    let (a, b) = (opened(&mut board, LOBBY, A, 1), opened(&mut board, LOBBY, B, 2));
    post(&mut board, LOBBY, "guest_1", &joining(A, true)).unwrap();
    post(&mut board, LOBBY, "guest_2", &joining(B, true)).unwrap();
    // The network dropped: A's page opens its line again. The old line
    // ends; the room hears A is out of the call, and the new line is
    // told who is in it, before its page joins again.
    let (again, note) = join::open(&mut board, LOBBY, A, &guest(1), 5).unwrap();
    assert_ne!(again, a);
    assert_eq!(note, [Note { to: a, tell: Tell::End }, roster(again, &[(B, 2)]), roster(b, &[(B, 2)]), Note { to: again, tell: Tell::First(listed(&[(B, 2)])) }]);
    // The old line closing afterwards closes nothing.
    assert!(leave::close(&mut board, a).is_empty());
    // It joins again, and stands where it stood: before B.
    let (_, answer) = post(&mut board, LOBBY, "guest_1", &joining(A, true)).unwrap();
    assert_eq!(answer.peer(), [A, B]);
}

#[test]
fn a_post_with_nothing_in_it_answers_who_is_in_the_call() {
    let mut board = board();
    opened(&mut board, LOBBY, A, 1);
    let (note, answer) = post(&mut board, LOBBY, "guest_1", &Post { peer: A.to_string(), ..Post::default() }).unwrap();
    assert!(note.is_empty() && answer.is_empty());
}
