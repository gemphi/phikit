//! A room's line as a caller holds it: what the browser's side does in
//! `puikit/app/phi-kit/src/call.ts`, in Rust. It opens the room's line,
//! an event stream, and hears who is in the call and what others send
//! it; and it posts to the same path to join, to leave and to send,
//! one post at a time and in order, so an offer never arrives after
//! its candidates.
//!
//! It speaks to the line as a page does, over HTTP, so the line may be
//! this window's own or one somewhere else: on the app's server it is
//! reached over TLS, at its place's path (`https://phiegg.com/wrt`,
//! net.rs). What it asks is the lines of `data/dial.tsv`, and the words
//! are `data/wire.tsv`'s.

use std::io::{BufRead, BufReader, Error, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use appkit::service::PTable;
use serde_json::{Map, Value, json};

use crate::comp::Wire;
use crate::data;
use crate::net::{self, Place, head, reach};

/// The kinds of ask `data/dial.tsv` has.
pub const STREAM: &str = "stream";
pub const POST: &str = "post";
/// The status of an ask that was taken.
pub const FINE: &str = "200";

/// What a caller hears on its line.
#[derive(Debug, Clone, PartialEq)]
pub enum Heard {
    /// Who is in the call now; and, the first time, the servers that
    /// help find a way.
    Roster(Vec<String>, Option<Vec<Value>>),
    /// What each of them calls themself, said just before the roster.
    Named(Vec<(String, String)>),
    /// What another caller sent this one.
    Signal(String, Value),
    /// The line refused what was asked, with that status.
    Refused(String),
    /// The line is gone.
    Down,
}

/// How the line is asked: the heads of `data/dial.tsv`, the words on
/// the wire, and what of `data/line.tsv` a caller needs.
#[derive(Debug, Clone)]
struct Dial {
    head: Vec<(String, String)>,
    wire: Wire,
    speak: String,
    path: String,
    host: String,
}

impl Dial {
    /// The head of one kind of ask, with its blanks filled.
    fn head(&self, kind: &str, fill: &[(&str, &str)]) -> String {
        let line = self.head.iter().filter(|(known, _)| known == kind).map(|(_, line)| {
            let filled = fill.iter().fold(line.clone(), |line, (name, value)| line.replace(&format!("{{{name}}}"), value));
            filled.replace("{speak}", &self.speak).replace("{path}", &self.path).replace("{host}", &self.host)
        });
        line.map(|line| format!("{line}\r\n")).collect::<String>() + "\r\n"
    }
}

/// A room's line, held by a caller.
pub struct Line {
    /// What this caller calls itself on the line.
    pub peer: String,
    heard: Receiver<Heard>,
    post: Sender<Value>,
    /// The socket under the line's stream: closed, the stream ends.
    socket: TcpStream,
    word: Wire,
}

impl Line {
    /// Open a room's line at a place, as a caller of that name. `Err`
    /// when the line is not there or refuses. A line on this machine is
    /// asked at its rooms' own path (`/call/...`); one at a place with a
    /// path of its own, under that path (`/wrt/...`).
    pub fn open(at: &str, key: &str, peer: &str, wait: Duration) -> std::io::Result<Line> {
        let line = PTable::read(data::LINE);
        let place = Place::read(at);
        let wait = place.patience(wait);
        let base = match place.base.is_empty() {
            true => format!("/{}", line.get("path")),
            false => place.base.clone(),
        };
        let asked = data::row(data::DIAL).into_iter().filter_map(|cell| Some((cell.first()?.to_string(), cell.get(1)?.to_string())));
        let dial = Dial { head: asked.collect(), wire: Wire::own(), speak: line.get("speak").to_string(), path: format!("{base}/{key}"), host: place.authority() };
        let mut wire = reach(&place, wait)?;
        wire.write_all(dial.head(STREAM, &[("name", &dial.wire.peer), ("peer", peer), ("cookie", "")]).as_bytes())?;
        let socket = wire.socket()?;
        let mut reader = BufReader::new(wire);
        let (status, said) = head(&mut reader)?;
        if status != FINE {
            return Err(Error::other(status));
        }
        // Who this caller is to the line: the cookie it was given.
        let given = said.iter().filter_map(|line| line.split_once(':')).find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"));
        let cookie = given.and_then(|(_, value)| value.split(';').next()).unwrap_or_default().trim().to_string();
        // A line may say nothing for as long as nobody comes or goes: it
        // is read without a limit, on the very wire the reader holds.
        reader.get_ref().set_read_timeout(None)?;
        let (tell, heard) = channel();
        let (post, asked) = channel::<Value>();
        let (events, words) = (tell.clone(), dial.wire.clone());
        std::thread::Builder::new().spawn(move || listen(reader, &words, &events))?;
        let asking = dial.clone();
        std::thread::Builder::new().spawn(move || asked.iter().for_each(|body| send(&asking, &place, &cookie, &body, wait, &tell)))?;
        Ok(Line { peer: peer.to_string(), heard, post, socket, word: dial.wire })
    }

    /// What was heard since this was last asked, in order.
    pub fn take(&self) -> Vec<Heard> {
        self.heard.try_iter().collect()
    }

    fn ask(&self, name: &str, body: Value) {
        let asked = Map::from_iter([(self.word.peer.clone(), json!(self.peer)), (name.to_string(), body)]);
        _ = self.post.send(Value::Object(asked));
    }

    /// Join the room's call, or leave it.
    pub fn join(&self, yes: bool) {
        self.ask(&self.word.join, json!(yes));
    }

    /// Send another caller something.
    pub fn send(&self, to: &str, signal: Value) {
        let one = Map::from_iter([(self.word.to.clone(), json!(to)), (self.word.signal.clone(), signal)]);
        self.ask(&self.word.send, json!([one]));
    }
}

impl Drop for Line {
    /// Letting the line go closes it: the line takes that as leaving.
    fn drop(&mut self) {
        _ = self.socket.shutdown(Shutdown::Both);
    }
}

/// Read the stream's events until it ends, and say each.
fn listen(reader: BufReader<net::Wire>, word: &Wire, tell: &Sender<Heard>) {
    let begin = format!("{} ", word.event);
    for line in reader.lines().map_while(Result::ok) {
        let Some(event) = line.strip_prefix(&begin).and_then(|text| serde_json::from_str::<Value>(text).ok()) else { continue };
        let peer = |list: &Value| list.as_array().into_iter().flatten().filter_map(|one| one.get(&word.peer)?.as_str()).map(str::to_string).collect::<Vec<_>>();
        let named = |one: &Value| Some((one.get(&word.peer)?.as_str()?.to_string(), one.get(&word.name).and_then(Value::as_str).unwrap_or_default().to_string()));
        let list = event.get(&word.roster).and_then(Value::as_array).map(|list| list.iter().filter_map(named).collect::<Vec<_>>());
        if list.is_some_and(|list| tell.send(Heard::Named(list)).is_err()) {
            return;
        }
        let heard = match (event.get(&word.roster), event.get(&word.from).and_then(Value::as_str), event.get(&word.signal)) {
            (Some(roster), ..) => Heard::Roster(peer(roster), event.get(&word.ice).and_then(Value::as_array).cloned()),
            (None, Some(from), Some(signal)) => Heard::Signal(from.to_string(), signal.clone()),
            _ => continue,
        };
        if tell.send(heard).is_err() {
            return;
        }
    }
    _ = tell.send(Heard::Down);
}

/// Post one thing to the room, and say so when it was refused.
fn send(dial: &Dial, place: &Place, cookie: &str, body: &Value, wait: Duration, tell: &Sender<Heard>) {
    let text = body.to_string();
    let asked = dial.head(POST, &[("cookie", cookie), ("length", &text.len().to_string())]) + &text;
    let answered = reach(place, wait).and_then(|mut wire| {
        wire.write_all(asked.as_bytes())?;
        let mut reader = BufReader::new(wire);
        let (status, _) = head(&mut reader)?;
        _ = reader.read_to_end(&mut Vec::new());
        Ok(status)
    });
    match answered {
        Ok(status) if status == FINE => {}
        Ok(status) => _ = tell.send(Heard::Refused(status)),
        Err(_) => _ = tell.send(Heard::Down),
    }
}
