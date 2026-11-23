//! The routes: what one wire is answered. They are the Node line's
//! (`puikit/app/phiora-server/src/call.mjs`), so the pages that spoke
//! to it speak to this one unchanged:
//!
//! * `GET /call/APP/ROOM?peer=ID`: the room's line, an event stream.
//!   Its first event is who is in the call and the ICE servers; a new
//!   roster follows each time someone joins or leaves, and a signal
//!   when a caller sends this page one. It stays open until the page
//!   goes away.
//! * `POST /call/APP/ROOM`: a peer, whether to join, and signals to
//!   send, from a page whose line is open.
//! * `OPTIONS`: a browser asking first whether it may call.

use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Map, Value};

use super::answer::{self, To};
use super::ask::{Ask, ORIGIN};
use super::hub::Hub;
use super::person::Asked;
use super::set::{ASK, JSON, STREAM};
use crate::comp::{Fault, Limit, Post};
use crate::entity::{Caller, Room};

/// The methods the routes know.
pub const GET: &str = "GET";
pub const POST: &str = "POST";
pub const OPTIONS: &str = "OPTIONS";
/// What an answer with nothing in it and one that is a stream answer.
pub const EMPTY: u16 = 204;
pub const FINE: u16 = 200;

/// Serve one wire: read what it asks and answer it. A room's line is
/// held open for as long as the page is there.
pub fn serve(hub: &Arc<Hub>, mut wire: TcpStream) {
    let (most, wait) = hub.limit(|limit| (limit.body, limit.gone));
    _ = wire.set_nodelay(true);
    _ = wire.set_read_timeout(Some(Duration::from_millis(wait)));
    let ask = match Ask::read(&mut wire, most) {
        Ok(ask) => ask,
        Err(fault) => return fault.into_iter().for_each(|fault| send(&mut wire, &answer::fail(&hub.set, fault, To::default()))),
    };
    let set = &hub.set;
    let named = ask.head(ORIGIN);
    let origin = Some(named).filter(|origin| set.allowed(origin));
    let to = To { origin, cookie: None };
    // A browser asks first; and a page that may not call may only read.
    match (ask.method.as_str(), origin, named.is_empty()) {
        (OPTIONS, Some(_), _) => return send(&mut wire, &answer::whole(set, EMPTY, &[JSON, ASK], "", to)),
        (OPTIONS, None, _) => return send(&mut wire, &answer::fail(set, Fault::Origin, to)),
        (GET, ..) | (_, Some(_), _) | (_, _, true) => {}
        _ => return send(&mut wire, &answer::fail(set, Fault::Origin, to)),
    }
    // Who the page belongs to: whoever a believed line in front says,
    // else a guest by this line's own cookie.
    let who = || set.told.caller(&ask).map_or_else(|| hub.guest(ask.cookie(&set.cookie)), |caller| (caller, None));
    // What Phiora answers besides a call: an app's document, which is
    // anyone's, and a person's own.
    if let Some(asked) = hub.person.asked(&ask.path) {
        let (caller, fresh) = match asked {
            Asked::App(_) => (Caller::default(), None),
            Asked::Me | Asked::State(_) => who(),
        };
        let to = To { origin, cookie: fresh.as_deref() };
        let answered = match hub.person.answer(&asked, &ask, &caller, &hub.limit(Limit::clone)) {
            Ok((status, Some(body))) => answer::json(set, status, &body, to),
            Ok((status, None)) => answer::whole(set, status, &[JSON], "", to),
            Err((fault, key)) => answer::about(set, fault, &key, to),
        };
        return send(&mut wire, &answered);
    }
    let part = ask.part();
    let key = match part.as_slice() {
        [path, app, room] if *path == set.path => hub.limit(|limit| Room::key(limit, app, room)),
        _ => None,
    };
    let Some(key) = key else { return send(&mut wire, &answer::fail(set, Fault::Route, to)) };
    let (caller, fresh) = who();
    let to = To { origin, cookie: fresh.as_deref() };
    match ask.method.as_str() {
        GET => stream(hub, wire, &key, ask.query(&set.wire.peer), &caller, to),
        POST => send(&mut wire, &post(hub, &key, &ask, &caller, to)),
        _ => send(&mut wire, &answer::fail(set, Fault::Route, to)),
    }
}

/// Write a whole answer and let the wire go.
fn send(wire: &mut TcpStream, answer: &[u8]) {
    _ = wire.write_all(answer).and_then(|_| wire.flush());
    _ = wire.shutdown(Shutdown::Write);
}

/// What a post is answered: who is in the call after it, or why not.
fn post(hub: &Hub, key: &str, ask: &Ask, caller: &Caller, to: To) -> Vec<u8> {
    let set = &hub.set;
    let body = match ask.body.is_empty() {
        true => Ok(Value::Object(Map::new())),
        false => serde_json::from_slice::<Value>(&ask.body),
    };
    let Ok(body) = body else { return answer::fail(set, Fault::Json, to) };
    let post = Post::read(&body, &set.wire, hub.limit(|limit| limit.send));
    match hub.post(key, &caller.user, &post) {
        Ok(roster) => answer::json(set, FINE, &Value::Object(Map::from_iter([(set.wire.roster.clone(), roster.json(&set.wire))])), to),
        Err(fault) => answer::fail(set, fault, to),
    }
}

/// A room's line. One thread writes what the page is told, as the hub
/// hands it over; this one waits for the page to go away, which is the
/// wire closing, and then closes the line.
fn stream(hub: &Arc<Hub>, mut wire: TcpStream, key: &str, peer: &str, caller: &Caller, to: To) {
    let (hold, told) = match hub.open(key, peer, caller) {
        Ok(open) => open,
        Err(fault) => return send(&mut wire, &answer::fail(&hub.set, fault, to)),
    };
    let head = format!("{}\r\n", answer::head(&hub.set, FINE, &[STREAM], to));
    let begun = wire.write_all(head.as_bytes()).and_then(|_| wire.flush()).and_then(|_| wire.try_clone());
    let Ok(mut out) = begun else { return hub.close(hold) };
    let writer = Arc::clone(hub);
    let write = move || {
        // It ends when the line loses its way, or the page takes no more.
        for text in told {
            if out.write_all(text.as_bytes()).and_then(|_| out.flush()).is_err() {
                break;
            }
            writer.seen(hold);
        }
        _ = out.shutdown(Shutdown::Both);
        writer.close(hold);
    };
    if std::thread::Builder::new().spawn(write).is_err() {
        return hub.close(hold);
    }
    // A page says nothing on its line: when the read ends, it is gone.
    _ = wire.set_read_timeout(None);
    let mut heard = [0u8; 256];
    while matches!(wire.read(&mut heard), Ok(count) if count > 0) {}
    hub.close(hold);
}
