//! What the rules read and say: the limits, the names, the refusals,
//! the words on the wire, the servers.

use serde_json::{Value, json};

use super::listed;
use crate::comp::{APP, Fault, Ice, Limit, PEER, Post, ROOM, Refusal, Tell, Wire};
use crate::data;
use crate::entity::Room;

#[test]
fn every_limit_is_a_row_of_the_table() {
    let limit = Limit::own();
    assert_eq!((limit.caller, limit.watch, limit.beat, limit.gone, limit.send), (8, 256, 20000, 60000, 64));
    // Another table, another call: nothing of it is in the code.
    let other = Limit::read("caller\t3\nbeat\t50", data::NAME);
    assert_eq!((other.caller, other.beat, other.watch), (3, 50, 0));
}

#[test]
fn a_name_is_what_its_row_lets_through() {
    let limit = Limit::own();
    assert!(limit.fits(PEER, "peer-aaaaaaaa") && limit.fits(PEER, "0f8fad5b-d9cb-469f-a165-70867728950e"));
    assert!(!limit.fits(PEER, "short") && !limit.fits(PEER, "peer.aaaaaaaa") && !limit.fits(PEER, &"a".repeat(65)));
    assert!(limit.fits(APP, "phiogy") && !limit.fits(APP, "phi.ogy") && !limit.fits(APP, ""));
    assert!(limit.fits(ROOM, "v_acrobirds-lobby") && limit.fits(ROOM, "show.tell") && !limit.fits(ROOM, "a/b"));
    assert!(!limit.fits("nothing", "peer-aaaaaaaa"));
    assert_eq!(Room::key(&limit, "phiogy", "lobby").as_deref(), Some("phiogy/lobby"));
    assert_eq!(Room::key(&limit, "phiogy", "lob by"), None);
}

#[test]
fn every_fault_has_a_status_a_word_and_a_sentence() {
    let refusal = Refusal::read(data::FAULT);
    let all = [Fault::Peer, Fault::Owner, Fault::Watch, Fault::Line, Fault::Caller, Fault::Origin, Fault::Route, Fault::Large, Fault::Json, Fault::Wire, Fault::App, Fault::Name, Fault::Unset, Fault::Empty];
    let bare: Vec<Fault> = all.into_iter().filter(|fault| matches!(refusal.of(*fault), (0, ..) | (_, "", _) | (_, _, ""))).collect();
    assert!(bare.is_empty(), "{bare:?}");
    // The ones a page reads are the Node line's.
    assert_eq!(refusal.of(Fault::Peer), (400, "bad_peer", "Name this page: ?peer=<id>."));
    assert_eq!(refusal.of(Fault::Line), (403, "no_line", "Open the room's line first."));
    assert_eq!((refusal.of(Fault::Caller).0, refusal.of(Fault::Watch).0, refusal.of(Fault::Owner).0), (409, 429, 403));
}

#[test]
fn what_a_page_is_told_is_spelled_as_the_browser_reads_it() {
    let (wire, ice) = (Wire::own(), Ice::read(data::ICE, None));
    let event = |tell: Tell| {
        let text = tell.text(&wire, &ice).expect("it is said");
        let body = text.strip_prefix("data: ").and_then(|rest| rest.strip_suffix("\n\n")).expect("an event of the stream").to_string();
        serde_json::from_str::<Value>(&body).expect("JSON")
    };
    assert_eq!(event(Tell::First(listed(&[]))), json!({ "roster": [], "ice": [] }));
    assert_eq!(event(Tell::Roster(listed(&[("peer-aaaaaaaa", 1)]))), json!({ "roster": [{ "peer": "peer-aaaaaaaa", "name": "Guest", "handle": null }] }));
    assert_eq!(event(Tell::Signal { from: "peer-aaaaaaaa".to_string(), body: json!({ "state": { "quiet": true } }) }), json!({ "from": "peer-aaaaaaaa", "signal": { "state": { "quiet": true } } }));
    assert_eq!(Tell::Beat.text(&wire, &ice).as_deref(), Some(": beat\n\n"));
    assert_eq!(Tell::End.text(&wire, &ice), None);
}

#[test]
fn a_post_is_read_as_a_page_writes_it() {
    let wire = Wire::own();
    let body = json!({ "peer": "peer-aaaaaaaa", "join": true, "send": [{ "to": "b", "signal": { "x": 1 } }, { "to": "b", "signal": "text" }, { "signal": {} }, { "to": "c", "signal": [1] }, { "to": "d", "signal": {} }] });
    let post = Post::read(&body, &wire, 4);
    // Of the first four, the two that name a page and carry something.
    assert_eq!((post.peer.as_str(), post.join), ("peer-aaaaaaaa", Some(true)));
    assert_eq!(post.send.iter().map(|signal| signal.to.as_str()).collect::<Vec<_>>(), ["b", "c"]);
    // To join is said with yes or no, and nothing else.
    assert_eq!(Post::read(&json!({ "peer": "p", "join": "yes" }), &wire, 4).join, None);
    assert_eq!(Post::read(&json!([]), &wire, 4), Post::default());
}

#[test]
fn the_servers_are_the_table_s_or_the_list_the_system_gives() {
    assert_eq!(Ice::read(data::ICE, None).json(), json!([]));
    let table = "urls\tusername\tcredential\nstun:one.test:3478\nturn:two.test:3478\tme\tsecret";
    assert_eq!(Ice::read(table, None).json(), json!([{ "urls": "stun:one.test:3478" }, { "urls": "turn:two.test:3478", "username": "me", "credential": "secret" }]));
    assert_eq!(Ice::read(table, Some(r#"[{"urls":"stun:given.test"}]"#)).json(), json!([{ "urls": "stun:given.test" }]));
    // What is given and is no list is not used.
    assert_eq!(Ice::read(data::ICE, Some("nonsense")).json(), json!([]));
}
