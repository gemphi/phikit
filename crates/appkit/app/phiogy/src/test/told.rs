//! A line with Phiora in front of it: who a page is, is what that line
//! says, when it knows the word; without the word nobody is believed.

use std::sync::{Arc, Mutex};

use appkit::service::PTable;
use serde_json::json;

use super::board;
use super::exchange::{ICE, Page, line};
use crate::data;
use crate::exchange::{Exchange, Set};

#[test]
fn a_page_is_who_a_believed_line_in_front_says() {
    let table = PTable::read(&format!("port\t0\n{}", data::LINE));
    let line = Exchange::open(Set::read(&table, Some(ICE), None).trust(Some("word")), Arc::new(Mutex::new(board()))).expect("the line opens");
    let at = line.at();
    let who = |key: &str| format!("X-Phi-Key: {key}\r\nX-Phi-User: u_ines\r\nX-Phi-Name: In%C3%A8s%20Valcourt\r\nX-Phi-Handle: ines.v\r\n");
    // With the word: named as told, and given no cookie of the line's.
    let mut known = Page::told(&at, "peer-aaaaaaaa", "demo/lobby", "", &who("word"));
    assert_eq!((known.status, known.cookie.as_str()), (200, ""));
    known.next();
    assert_eq!(known.say(json!({ "join": true })).0, 200);
    assert_eq!(known.next(), Some(json!({ "roster": [{ "peer": "peer-aaaaaaaa", "name": "In\u{e8}s Valcourt", "handle": "ines.v" }] })));
    // The same person on a second page may speak for the first: a page
    // belongs to who they are.
    let other = Page::told(&at, "peer-bbbbbbbb", "demo/lobby", "", &who("word"));
    assert_eq!(other.say(json!({ "peer": "peer-aaaaaaaa", "join": false })).0, 200);
    // Without the word nobody is believed: a guest, with a cookie.
    let mut guest = Page::told(&at, "peer-cccccccc", "demo/lobby", "", &who("guess"));
    assert!(guest.cookie.starts_with("phi_sid="), "{:?}", guest.head);
    guest.next();
    guest.say(json!({ "join": true }));
    assert_eq!(guest.next().unwrap()["roster"][0]["name"], json!("Guest"));
}

#[test]
fn a_line_given_no_word_believes_nobody() {
    let line = line();
    let at = line.at();
    let page = Page::told(&at, "peer-aaaaaaaa", "demo/lobby", "", "X-Phi-Key: \r\nX-Phi-User: u_ines\r\nX-Phi-Name: Ines\r\n");
    assert!(page.cookie.starts_with("phi_sid="), "{:?}", page.head);
}
