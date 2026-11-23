//! Phiora, asked as a page asks it: the server that keeps every app's
//! records and every person's own documents
//! (`puikit/app/phiora-server`). The window asks it first for its own
//! document, and reads its files where Phiora does not answer, as the
//! web does (`loadApp` in `puikit/app/phi-kit/src/data.ts`); and what a
//! person writes, marks and pins is kept there under their session, as
//! a page keeps it (`persisted` in `api.ts`).
//!
//! It is asked over plain HTTP, which is what a Phiora on this machine
//! or beside it speaks; one that is reached only over a secured line is
//! not asked, and the files answer. Where it is, how long it is waited
//! for and every word of an ask are `data/phiora.tsv`.

use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

use appkit::service::PTable;

use crate::data;
use crate::peer::{head, reach};

/// The kinds of ask `data/phiora.tsv` has.
const GET: &str = "get";
const PUT: &str = "put";
const POST: &str = "post";

/// Phiora at a place.
#[derive(Debug, Clone)]
pub struct Phiora {
    /// Where it is: a name and a port.
    pub at: String,
    /// Who is asking, as Phiora knows them: the session it gave.
    pub cookie: String,
    word: PTable,
    line: Vec<(String, String)>,
    speak: String,
}

/// What Phiora answered: its status, the lines of its head, its words.
type Answer = (String, Vec<String>, String);

impl Phiora {
    /// Phiora where the command line said, or the system's setting, or
    /// `data/phiora.tsv`, in that order. Nobody, when the place is the
    /// word for none.
    pub fn own(given: Option<String>) -> Option<Phiora> {
        let word = PTable::read(data::PHIORA);
        let set = std::env::var(word.get("given")).ok().filter(|place| !place.trim().is_empty());
        let place = given.or(set).unwrap_or_else(|| word.get("at").to_string());
        let at = place.trim().strip_prefix(word.get("scheme")).unwrap_or(place.trim()).trim_end_matches('/');
        (!at.is_empty() && at != word.get("none")).then(|| Phiora::at(at))
    }

    /// Phiora at that place, asked by nobody it knows yet.
    pub fn at(at: &str) -> Phiora {
        let line = data::row(data::PHIORA).into_iter().filter(|cell| [GET, PUT, POST].contains(&cell[0])).filter_map(|cell| Some((cell[0].to_string(), cell.get(1)?.to_string())));
        Phiora { at: at.to_string(), cookie: String::new(), word: PTable::read(data::PHIORA), line: line.collect(), speak: PTable::read(data::LINE).get("speak").to_string() }
    }

    fn time(&self, name: &str) -> Duration {
        Duration::from_millis(self.word.get(name).parse().unwrap_or_default())
    }

    /// Ask one thing, and read all of the answer. Nothing when Phiora is
    /// not there, or takes longer than it is waited for.
    fn ask(&self, kind: &str, path: &str, body: &str) -> Option<Answer> {
        let fill = |line: &str| line.replace("{path}", path).replace("{speak}", &self.speak).replace("{host}", &self.at).replace("{agent}", self.word.get("agent")).replace("{cookie}", &self.cookie).replace("{length}", &body.len().to_string());
        let lines = self.line.iter().filter(|(known, _)| known == kind).map(|(_, line)| fill(line) + "\r\n");
        let asked = lines.collect::<String>() + "\r\n" + body;
        let wire = reach(&self.at, self.time("wait")).ok()?;
        wire.set_read_timeout(Some(self.time("answer"))).ok()?;
        (&wire).write_all(asked.as_bytes()).ok()?;
        let mut reader = BufReader::new(wire);
        let (status, said) = head(&mut reader).ok()?;
        let length = named(&said, self.word.get("length")).and_then(|length| length.parse::<usize>().ok());
        let body = match (named(&said, self.word.get("pieces")), length) {
            (Some(how), _) if how.contains(self.word.get("piece")) => pieces(&mut reader)?,
            (_, Some(length)) => {
                let mut body = vec![0; length];
                reader.read_exact(&mut body).ok()?;
                body
            }
            _ => {
                let mut body = Vec::new();
                reader.read_to_end(&mut body).ok()?;
                body
            }
        };
        Some((status, said, String::from_utf8(body).ok()?))
    }

    /// What Phiora answers a path with. Nothing when it is not there,
    /// when it refuses, and when it takes longer than it is waited for.
    pub fn get(&self, path: &str) -> Option<String> {
        self.ask(GET, path, "").filter(|(status, ..)| status == self.word.get("fine")).map(|(.., body)| body)
    }

    /// An app's own document, as Phiora composes it from its records.
    pub fn app(&self, key: &str) -> Option<String> {
        self.get(&self.word.get("app").replace("{key}", key))
    }

    /// Be somebody to Phiora: with the session it gave before, or a new
    /// one, which it gives whoever asks who they are. `false` when it
    /// does not answer.
    pub fn enter(&mut self) -> bool {
        let Some((status, said, _)) = self.ask(GET, self.word.get("me"), "") else { return false };
        // A session it no longer knows is answered with another.
        let given = said.iter().filter_map(|line| line.split_once(':')).find(|(name, _)| name.trim().eq_ignore_ascii_case(self.word.get("session")));
        given.and_then(|(_, value)| value.split(';').next()).into_iter().for_each(|cookie| self.cookie = cookie.trim().to_string());
        status == self.word.get("fine") && !self.cookie.is_empty()
    }

    /// Who the one asking is, as Phiora says it.
    pub fn me(&self) -> Option<String> {
        self.get(self.word.get("me"))
    }

    /// One of the person's own documents; nothing for one never kept.
    pub fn state(&self, name: &str) -> Option<String> {
        self.get(&self.word.get("state").replace("{name}", name))
    }

    /// Keep one of the person's own documents. `false` when Phiora did
    /// not take it.
    pub fn keep(&self, name: &str, document: &str) -> bool {
        let taken = self.ask(PUT, &self.word.get("state").replace("{name}", name), document);
        taken.is_some_and(|(status, ..)| [self.word.get("fine"), self.word.get("kept")].contains(&status.as_str()))
    }

    /// Ask for something to be done, and hear the answer: whether
    /// Phiora did it, and what it said. Nothing when it is not there.
    pub fn said(&self, path: &str, body: &str) -> Option<(bool, String)> {
        self.ask(POST, path, body).map(|(status, _, said)| (status == self.word.get("fine"), said))
    }

    /// Ask for something to be done: a body sent to a path. `false`
    /// when Phiora did not do it.
    pub fn post(&self, path: &str, body: &str) -> bool {
        self.ask(POST, path, body).is_some_and(|(status, ..)| status == self.word.get("fine"))
    }
}

/// What a line of a head says under a name, in small letters.
fn named(said: &[String], name: &str) -> Option<String> {
    let found = said.iter().filter_map(|line| line.split_once(':')).find(|(known, _)| known.trim().eq_ignore_ascii_case(name));
    found.map(|(_, value)| value.trim().to_ascii_lowercase())
}

/// A body that comes in pieces: each is its length on a line, in
/// sixteens, then that many bytes; a piece of no length ends it.
fn pieces(reader: &mut impl BufRead) -> Option<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        let size = usize::from_str_radix(line.split(';').next()?.trim(), 16).ok()?;
        if size == 0 {
            return Some(body);
        }
        let from = body.len();
        body.resize(from + size, 0);
        reader.read_exact(&mut body[from..]).ok()?;
        // The end of the piece's own line.
        reader.read_line(&mut line).ok()?;
    }
}
