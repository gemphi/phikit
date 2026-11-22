//! This window as a caller: the rule of who gives way, with no network
//! and nothing of WebRTC in it; and a call held over a real line.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

use super::board;
use crate::board::lock;
use crate::peer::{Call, Engine, Far, Maker, Mesh, Out, Stage, Stand};

/// Callers with no line between them: what one sends is handed to the
/// one it is for, until nobody has more to say.
struct Room {
    caller: Vec<(String, Mesh, Stand)>,
}

impl Room {
    /// Those callers, all in the call.
    fn of(name: &[&str]) -> Room {
        let mut room = Room { caller: name.iter().map(|name| (name.to_string(), Mesh::new(name), Stand::default())).collect() };
        room.roster(name);
        room
    }

    /// Tell everyone who is in the call, and let what follows play out.
    fn roster(&mut self, name: &[&str]) -> Vec<(String, String, Value)> {
        let roster: Vec<String> = name.iter().map(|name| name.to_string()).collect();
        let mut sent = Vec::new();
        for (from, mesh, engine) in &mut self.caller {
            sent.extend(mesh.sync(&roster, engine).into_iter().map(|(to, body)| (from.clone(), to, body)));
        }
        self.play(sent)
    }

    /// Hand each thing sent to whom it is for and take what each then
    /// makes, until all is quiet. Everything that was sent, in order.
    fn play(&mut self, mut sent: Vec<(String, String, Value)>) -> Vec<(String, String, Value)> {
        let (mut all, mut quiet) = (Vec::new(), 0);
        for _ in 0..64 {
            for (from, to, body) in sent.drain(..) {
                self.caller.iter_mut().filter(|(name, ..)| *name == to).for_each(|(_, mesh, _)| mesh.hear(&from, &body));
                all.push((from, to, body));
            }
            for (from, mesh, _) in &mut self.caller {
                let out: Out = mesh.turn();
                sent.extend(out.into_iter().map(|(to, body)| (from.clone(), to, body)));
            }
            // A link asked to offer has its offer a turn later: all is
            // quiet when nothing was sent twice running.
            quiet = match sent.is_empty() {
                true => quiet + 1,
                false => 0,
            };
            if quiet == 2 {
                return all;
            }
        }
        panic!("the callers never stopped talking: {sent:?}");
    }

    fn far(&self, name: &str) -> Vec<Far> {
        self.caller.iter().find(|(known, ..)| known == name).map(|(_, mesh, _)| mesh.far()).unwrap_or_default()
    }

    fn back(&self, name: &str) -> usize {
        self.caller.iter().find(|(known, ..)| known == name).map_or(0, |(.., engine)| engine.back.load(Ordering::SeqCst))
    }
}

#[test]
fn when_both_offer_at_once_the_lesser_name_gives_way() {
    // Both come with something to send, so both offer at once.
    let room = Room::of(&["peer-aaaa", "peer-bbbb"]);
    // Both end connected, each to the other.
    let stage = |name: &str| room.far(name).iter().map(|far| (far.peer.clone(), far.stage)).collect::<Vec<_>>();
    assert_eq!(stage("peer-aaaa"), [("peer-bbbb".to_string(), Stage::Connected)]);
    assert_eq!(stage("peer-bbbb"), [("peer-aaaa".to_string(), Stage::Connected)]);
    // The lesser name took its offer back, once; the greater never did.
    assert_eq!((room.back("peer-aaaa"), room.back("peer-bbbb")), (1, 0));
}

#[test]
fn what_callers_say_is_spelled_as_the_browser_s_side_spells_it() {
    let mut room = Room { caller: ["peer-aaaa", "peer-bbbb"].iter().map(|name| (name.to_string(), Mesh::new(name), Stand::default())).collect() };
    let said = room.roster(&["peer-aaaa", "peer-bbbb"]);
    let from = |name: &str| said.iter().filter(|(from, ..)| from == name).map(|(_, _, body)| body.clone()).collect::<Vec<_>>();
    // Each begins by saying how it is, then offers.
    let b = from("peer-bbbb");
    assert_eq!(b[0], json!({ "state": { "quiet": false, "screen": "screen" } }));
    assert_eq!(b[1], json!({ "description": { "type": "offer", "sdp": "offer 1" } }));
    // The lesser name answers the greater's offer, and the greater lets
    // the lesser's pass: no answer to it is ever sent.
    let a = from("peer-aaaa");
    assert!(a.contains(&json!({ "description": { "type": "answer", "sdp": "answer to offer 1" } })), "{a:?}");
    assert!(!b.iter().any(|body| body["description"]["sdp"] == "answer to offer 1"), "{b:?}");
}

#[test]
fn every_caller_is_connected_to_every_other_and_let_go_when_they_leave() {
    let mut room = Room::of(&["peer-aaaa", "peer-bbbb", "peer-cccc"]);
    for name in ["peer-aaaa", "peer-bbbb", "peer-cccc"] {
        let far = room.far(name);
        assert!(far.len() == 2 && far.iter().all(|far| far.stage == Stage::Connected), "{name}: {far:?}");
    }
    // One leaves: the two left keep each other and let go of it; the
    // one who left, being out of the call, keeps nobody.
    room.roster(&["peer-aaaa", "peer-cccc"]);
    let peer = |name: &str| room.far(name).iter().map(|far| far.peer.clone()).collect::<Vec<_>>();
    assert_eq!((peer("peer-aaaa"), peer("peer-cccc"), peer("peer-bbbb")), (vec!["peer-cccc".to_string()], vec!["peer-aaaa".to_string()], vec![]));
}

#[test]
fn a_caller_that_mutes_tells_the_others() {
    let mut room = Room::of(&["peer-aaaa", "peer-bbbb"]);
    let (_, mesh, engine) = &mut room.caller[0];
    let sent: Vec<(String, String, Value)> = mesh.hush(true, engine).into_iter().map(|(to, body)| ("peer-aaaa".to_string(), to, body)).collect();
    assert_eq!(sent, [("peer-aaaa".to_string(), "peer-bbbb".to_string(), json!({ "state": { "quiet": true, "screen": "screen" } }))]);
    room.play(sent);
    assert!(room.far("peer-bbbb")[0].quiet && !room.far("peer-aaaa")[0].quiet);
}

#[test]
fn a_name_this_window_takes_never_gives_way_to_a_browser() {
    let name = crate::peer::name().expect("at a desk there is chance to be had");
    // A browser's page names itself with digits and the letters a to f.
    assert!(name.as_str() > "ffffffff-ffff-4fff-bfff-ffffffffffff", "{name}");
    assert!(crate::comp::Limit::own().fits(crate::comp::PEER, &name), "{name}");
    assert_ne!(Some(name), crate::peer::name());
}

/// What makes a stand-in engine on a call's own thread.
fn stand() -> Maker {
    Box::new(|| Box::new(Stand::default()) as Box<dyn Engine>)
}

/// Who the board has in a room's call. The board is held once: the
/// line changes it from other threads.
pub(super) fn calling(board: &Mutex<crate::Board>, key: &str) -> Vec<String> {
    let board = lock(board);
    let roster = board.room(key).map(|room| board.roster(room));
    roster.map(|roster| roster.peer().iter().map(|peer| peer.to_string()).collect()).unwrap_or_default()
}

/// Wait until something is so, a few seconds at most.
pub(super) fn until(what: &str, so: impl Fn() -> bool) {
    for _ in 0..400 {
        if so() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("never: {what}");
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn two_windows_join_a_room_over_the_line_and_connect() {
    let board = Arc::new(Mutex::new(board()));
    let line = super::exchange::line_on(Arc::clone(&board));
    let at = line.at();
    let a = Call::start(&at, "phiogy/lora", "phiogy-aaaaaaaa", stand()).expect("the line is there");
    let b = Call::start(&at, "phiogy/lora", "phiogy-bbbbbbbb", stand()).expect("the line is there");
    // Each is in the call, connected to the other, by the same routes a
    // page uses: the board has them as two callers.
    let connected = |call: &Call, other: &str| call.seen().joined && call.seen().far.iter().any(|far| far.peer == other && far.stage == Stage::Connected);
    until("both connected", || connected(&a, "phiogy-bbbbbbbb") && connected(&b, "phiogy-aaaaaaaa"));
    assert_eq!(calling(&board, "phiogy/lora"), ["phiogy-aaaaaaaa", "phiogy-bbbbbbbb"]);
    assert_eq!(a.seen().engine, "stand-in");
    // One mutes, and the other is told through the line.
    a.hush(true);
    until("the other hears of the mute", || b.seen().far.iter().any(|far| far.quiet));
    // One leaves: the line drops it, and the other lets it go.
    drop(a);
    until("the one left is alone", || b.seen().far.is_empty() && lock(&board).open() == [("phiogy/lora".to_string(), 1, 1)]);
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn a_call_that_is_full_refuses_this_window_and_says_so() {
    let board = Arc::new(Mutex::new(super::bound("caller\t1")));
    let line = super::exchange::line_on(Arc::clone(&board));
    let at = line.at();
    let first = Call::start(&at, "phiogy/lora", "phiogy-aaaaaaaa", stand()).expect("the line is there");
    until("the first is in", || first.seen().joined);
    let second = Call::start(&at, "phiogy/lora", "phiogy-bbbbbbbb", stand()).expect("the line is there");
    until("the second is refused", || second.seen().over.as_deref() == Some("409"));
    assert!(!second.seen().joined && first.seen().joined);
    // A name the line does not take, and a line that is not there.
    assert!(Call::start(&at, "phiogy/lora", "short", stand()).is_err());
    assert!(Call::start("localhost:9", "phiogy/lora", "phiogy-cccccccc", stand()).is_err());
}
