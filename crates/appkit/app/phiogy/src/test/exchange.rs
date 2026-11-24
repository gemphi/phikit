//! The line over a real socket on this machine: the three cases of the
//! Node line's tests (`puikit/app/phiora-server/test/call.test.mjs`),
//! asked the way a page asks them, and what a browser needs besides.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use appkit::service::PTable;
use serde_json::{Value, json};

use super::board;
use crate::board::Board;
use crate::data;
use crate::exchange::{Exchange, Set};

const ORIGIN: &str = "http://localhost:3118";
pub(super) const ICE: &str = r#"[{"urls":"stun:stun.test:3478"}]"#;

/// The line on a port the system chooses, told of one ICE server.
pub(super) fn line() -> Exchange {
    line_on(Arc::new(Mutex::new(board())))
}

/// The same line, on a board somebody else reads as well.
pub(crate) fn line_on(board: Arc<Mutex<Board>>) -> Exchange {
    let table = PTable::read(&format!("port\t0\n{}", data::LINE));
    Exchange::open(Set::read(&table, Some(ICE), None), board).expect("the line opens")
}

/// Ask the line something, and read the head of its answer: the wire,
/// the status and the head's lines.
fn ask(at: &str, request: &str) -> (BufReader<TcpStream>, u16, Vec<String>) {
    let wire = TcpStream::connect(at).expect("the line is there");
    wire.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    (&wire).write_all(request.as_bytes()).unwrap();
    let mut reader = BufReader::new(wire);
    let mut head = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("the line answers");
        match line.trim() {
            "" => break,
            line => head.push(line.to_string()),
        }
    }
    let status = head[0].split_whitespace().nth(1).and_then(|code| code.parse().ok()).unwrap_or_default();
    (reader, status, head)
}

/// What a line of the head says under a name.
fn said<'a>(head: &'a [String], name: &str) -> &'a str {
    head.iter().filter_map(|line| line.split_once(": ")).find(|(known, _)| *known == name).map_or("", |(_, value)| value)
}

/// A page: its cookie, its name, its room's line and posts to the room.
pub(crate) struct Page {
    at: String,
    room: String,
    peer: String,
    pub(super) cookie: String,
    /// More lines of the head it sends each time: who a line in front
    /// says it belongs to.
    more: String,
    pub(super) status: u16,
    pub(super) head: Vec<String>,
    wire: BufReader<TcpStream>,
}

impl Page {
    /// Open a room's line as a page of that name, with that cookie or
    /// none.
    pub(crate) fn open(at: &str, peer: &str, room: &str, cookie: &str) -> Page {
        Page::told(at, peer, room, cookie, "")
    }

    /// The same, sending more lines in its head each time.
    pub(super) fn told(at: &str, peer: &str, room: &str, cookie: &str, more: &str) -> Page {
        let (wire, status, head) = ask(at, &format!("GET /call/{room}?peer={peer} HTTP/1.1\r\nHost: {at}\r\nOrigin: {ORIGIN}\r\nCookie: {cookie}\r\n{more}\r\n"));
        let given = said(&head, "set-cookie").split(';').next().unwrap_or_default().to_string();
        Page { at: at.to_string(), room: room.to_string(), peer: peer.to_string(), cookie: given, more: more.to_string(), status, head, wire }
    }

    /// The next event of the line; nothing when the line ended.
    pub(crate) fn next(&mut self) -> Option<Value> {
        loop {
            let mut line = String::new();
            match self.wire.read_line(&mut line) {
                Ok(0) | Err(_) => return None,
                Ok(_) => {}
            }
            if let Some(event) = line.trim().strip_prefix("data: ") {
                return serde_json::from_str(event).ok();
            }
        }
    }

    /// Post to the room as this page, with this page's cookie or
    /// another's: the status and what was answered.
    fn post(&self, body: Value, cookie: &str) -> (u16, Value) {
        let mut body = body;
        if body.get("peer").is_none() {
            body["peer"] = json!(self.peer);
        }
        let text = body.to_string();
        let request = format!("POST /call/{} HTTP/1.1\r\nHost: {}\r\nOrigin: {ORIGIN}\r\nCookie: {cookie}\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{text}", self.room, self.at, self.more, text.len());
        let (mut wire, status, _) = ask(&self.at, &request);
        let mut answer = String::new();
        wire.read_to_string(&mut answer).unwrap();
        (status, serde_json::from_str(&answer).unwrap_or_default())
    }

    pub(crate) fn say(&self, body: Value) -> (u16, Value) {
        self.post(body, &self.cookie)
    }

    /// The page goes away.
    pub(crate) fn close(self) {
        _ = self.wire.get_ref().shutdown(Shutdown::Both);
    }
}

#[test]
fn two_pages_in_a_room_find_each_other_and_trade_signals() {
    let line = line();
    let at = line.at();
    let (mut a, mut b) = (Page::open(&at, "peer-aaaaaaaa", "demo/lobby", ""), Page::open(&at, "peer-bbbbbbbb", "demo/lobby", ""));
    assert_eq!(a.status, 200);
    assert!(said(&a.head, "content-type").contains("text/event-stream"), "{:?}", a.head);
    assert_eq!(a.next(), Some(json!({ "roster": [], "ice": [{ "urls": "stun:stun.test:3478" }] })));
    assert_eq!(b.next().unwrap()["roster"], json!([]));
    // A starts a call: B, who has only the room open, hears of it.
    assert_eq!(a.say(json!({ "join": true })).0, 200);
    let calling = json!({ "roster": [{ "peer": "peer-aaaaaaaa", "name": "Guest", "handle": null }] });
    assert_eq!((a.next(), b.next()), (Some(calling.clone()), Some(calling.clone())));
    // A signal to someone who has not joined goes nowhere; once both
    // are in, it arrives, in the order sent.
    a.say(json!({ "send": [{ "to": "peer-bbbbbbbb", "signal": { "lost": true } }] }));
    b.say(json!({ "join": true }));
    assert_eq!(a.next().unwrap()["roster"].as_array().map(Vec::len), Some(2));
    assert_eq!(b.next().unwrap()["roster"].as_array().map(Vec::len), Some(2));
    let (offer, candidate) = (json!({ "description": { "type": "offer", "sdp": "v=0" } }), json!({ "candidate": { "candidate": "c1" } }));
    a.say(json!({ "send": [{ "to": "peer-bbbbbbbb", "signal": offer }, { "to": "peer-bbbbbbbb", "signal": candidate }] }));
    assert_eq!(b.next(), Some(json!({ "from": "peer-aaaaaaaa", "signal": offer })));
    assert_eq!(b.next(), Some(json!({ "from": "peer-aaaaaaaa", "signal": candidate })));
    // B's page goes away: A's roster no longer holds them.
    b.close();
    assert_eq!(a.next(), Some(calling));
    let (status, answer) = a.say(json!({ "join": false }));
    assert_eq!((status, answer), (200, json!({ "roster": [] })));
    assert_eq!(a.next(), Some(json!({ "roster": [] })));
    a.close();
}

#[test]
fn a_peer_belongs_to_whoever_opened_its_line() {
    let line = line();
    let at = line.at();
    let (mut a, mut other) = (Page::open(&at, "peer-cccccccc", "demo/den", ""), Page::open(&at, "peer-dddddddd", "demo/den", ""));
    _ = (a.next(), other.next());
    assert_ne!(a.cookie, other.cookie);
    let (status, answer) = a.post(json!({ "join": true }), &other.cookie);
    assert_eq!((status, answer["error"].as_str()), (403, Some("no_line")));
    assert_eq!(other.say(json!({ "peer": "peer-eeeeeeee", "join": true })).0, 403);
    assert_eq!(Page::open(&at, "short", "demo/den", "").status, 400);
    assert_eq!(Page::open(&at, "peer-cccccccc", "demo/den", &other.cookie).status, 403);
    // A page that comes back with its cookie is who it was.
    assert_eq!(a.say(json!({ "join": true })).0, 200);
    a.close();
    other.close();
}

#[test]
fn a_call_holds_eight() {
    let line = line();
    let at = line.at();
    let page: Vec<Page> = (0..9).map(|index| Page::open(&at, &format!("peer-full-{index}xxx"), "demo/hall", "")).collect();
    let got: Vec<u16> = page.iter().map(|page| page.say(json!({ "join": true })).0).collect();
    assert_eq!(got, [200, 200, 200, 200, 200, 200, 200, 200, 409]);
    page.into_iter().for_each(Page::close);
}

#[test]
fn a_browser_is_answered_what_it_needs_to_call_from_another_port() {
    let line = line();
    let at = line.at();
    // It asks first, and is told it may, with its cookie.
    let (_, status, head) = ask(&at, &format!("OPTIONS /call/demo/lobby HTTP/1.1\r\nHost: {at}\r\nOrigin: {ORIGIN}\r\n\r\n"));
    assert_eq!((status, said(&head, "access-control-allow-origin"), said(&head, "access-control-allow-credentials")), (204, ORIGIN, "true"));
    assert!(said(&head, "access-control-allow-methods").contains("POST") && said(&head, "access-control-allow-headers").contains("content-type"));
    // A page from somewhere else may not ask, nor post.
    let (_, status, _) = ask(&at, &format!("OPTIONS /call/demo/lobby HTTP/1.1\r\nHost: {at}\r\nOrigin: http://elsewhere.test\r\n\r\n"));
    assert_eq!(status, 403);
    let (_, status, head) = ask(&at, &format!("POST /call/demo/lobby HTTP/1.1\r\nHost: {at}\r\nOrigin: http://elsewhere.test\r\nContent-Length: 2\r\n\r\n{{}}"));
    assert_eq!((status, said(&head, "access-control-allow-origin")), (403, ""));
    // A room's line names the page that may read it, and a new guest.
    let page = Page::open(&at, "peer-aaaaaaaa", "demo/lobby", "");
    assert_eq!(said(&page.head, "access-control-allow-origin"), ORIGIN);
    assert!(page.cookie.starts_with("phi_sid=") && said(&page.head, "set-cookie").contains("HttpOnly"), "{:?}", page.head);
    // Nothing else answers, and what is not JSON is not read.
    let (_, status, _) = ask(&at, &format!("GET /nothing HTTP/1.1\r\nHost: {at}\r\n\r\n"));
    assert_eq!(status, 404);
    let (_, status, _) = ask(&at, &format!("POST /call/demo/lobby HTTP/1.1\r\nHost: {at}\r\nCookie: {}\r\nContent-Length: 4\r\n\r\nnope", page.cookie));
    assert_eq!(status, 400);
    page.close();
}

#[test]
fn a_line_that_is_closed_ends_every_page_s_line() {
    let line = line();
    let at = line.at();
    let mut page = Page::open(&at, "peer-aaaaaaaa", "demo/lobby", "");
    assert!(page.next().is_some());
    line.close();
    assert_eq!(page.next(), None);
}

#[test]
fn the_line_listens_on_every_address_its_name_stands_for() {
    use std::net::ToSocketAddrs;
    let line = line();
    let at = line.at();
    // It says where it is as a page writes it: the name, and the port.
    let (name, port) = at.rsplit_once(':').expect("a name and a port");
    assert_eq!(name, PTable::read(data::LINE).get("host"));
    let port: u16 = port.parse().expect("a port");
    // A page that tries the old address first and one that tries the
    // new one are both answered, on the one port.
    let every: Vec<String> = (name, port).to_socket_addrs().expect("the name is known").map(|at| at.to_string()).collect();
    assert!(!every.is_empty());
    for at in &every {
        let (_, status, _) = ask(at, &format!("GET /nothing HTTP/1.1\r\nHost: {at}\r\n\r\n"));
        assert_eq!(status, 404, "{at}");
    }
}

#[test]
fn a_line_listens_at_the_name_it_is_given_so_another_machine_reaches_it() {
    // `--host` stands a row before the table's own, as `--port` does.
    let line = Exchange::open(Set::own("host\t127.0.0.1\nport\t0\n"), crate::Phiogy::board()).expect("it listens");
    let at = line.at();
    assert!(at.starts_with("127.0.0.1:"), "{at}");
    let (_, status, _) = ask(&at, &format!("GET /nothing HTTP/1.1\r\nHost: {at}\r\n\r\n"));
    assert_eq!(status, 404);
}
