//! Phiora, asked as a page asks it: the server that keeps every app's
//! records and every person's own documents
//! (`puikit/app/phiora-server`). The window asks it first for its own
//! document, and reads its files where Phiora does not answer, as the
//! web does (`loadApp` in `puikit/app/phi-kit/src/data.ts`); and what a
//! person writes, marks and pins is kept there under their session, as
//! a page keeps it (`persisted` in `api.ts`).
//!
//! It is asked over HTTP: plainly on this machine, over TLS on the
//! app's server (`https://phiegg.com/api`, net.rs), where everything is
//! asked under its place's path. Where it is, how long it is waited for
//! and every word of an ask are `data/phiora.tsv`.

use std::io::{BufReader, Write};
use std::time::Duration;

use appkit::service::PTable;

use crate::data;
use crate::net::{self, Place, head, reach};

/// The kinds of ask `data/phiora.tsv` has a head for: it reads (`Get`), keeps
/// a document (`Put`), asks for something to be done or a record made
/// (`Post`), changes a record (`Patch`) or takes one away (`Delete`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Get,
    Put,
    Post,
    Patch,
    Delete,
}

impl Verb {
    pub const ALL: [Verb; 5] = [Verb::Get, Verb::Put, Verb::Post, Verb::Patch, Verb::Delete];

    /// The name of its rows in `data/phiora.tsv`.
    fn row(self) -> &'static str {
        match self {
            Verb::Get => "get",
            Verb::Put => "put",
            Verb::Post => "post",
            Verb::Patch => "patch",
            Verb::Delete => "delete",
        }
    }
}

/// Phiora at a place.
#[derive(Debug, Clone)]
pub struct Phiora {
    /// Where it is, as it was given (`localhost:4300`,
    /// `https://phiegg.com/api`).
    pub at: String,
    place: Place,
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
    /// on the app's server, in that order. Nobody, when the place is the
    /// word for none.
    pub fn own(given: Option<String>) -> Option<Phiora> {
        let word = PTable::read(data::PHIORA);
        let set = std::env::var(word.get("given")).ok().filter(|place| !place.trim().is_empty());
        let place = given.or(set).unwrap_or_else(|| crate::Phiogy::server("server.phiora"));
        let at = place.trim().trim_end_matches('/');
        (!at.is_empty() && at != word.get("none")).then(|| Phiora::at(at))
    }

    /// Phiora at that place, asked by nobody it knows yet.
    pub fn at(at: &str) -> Phiora {
        let line = data::row(data::PHIORA).into_iter().filter(|cell| Verb::ALL.iter().any(|verb| verb.row() == cell[0])).filter_map(|cell| Some((cell[0].to_string(), cell.get(1)?.to_string())));
        Phiora { at: at.trim_end_matches('/').to_string(), place: Place::read(at), cookie: String::new(), word: PTable::read(data::PHIORA), line: line.collect(), speak: PTable::read(data::LINE).get("speak").to_string() }
    }

    /// How long it is waited for: a Phiora over the internet the longer.
    fn time(&self, name: &str) -> Duration {
        self.place.patience(Duration::from_millis(self.word.get(name).parse().unwrap_or_default()))
    }

    /// Ask one thing, and read all of the answer. Nothing when Phiora is
    /// not there, or takes longer than it is waited for.
    fn ask(&self, verb: Verb, path: &str, body: &str) -> Option<Answer> {
        let kind = verb.row();
        let path = format!("{}{path}", self.place.base);
        let fill = |line: &str| line.replace("{path}", &path).replace("{speak}", &self.speak).replace("{host}", &self.place.authority()).replace("{agent}", self.word.get("agent")).replace("{cookie}", &self.cookie).replace("{length}", &body.len().to_string());
        let lines = self.line.iter().filter(|(known, _)| known == kind).map(|(_, line)| fill(line) + "\r\n");
        let asked = lines.collect::<String>() + "\r\n" + body;
        let mut wire = reach(&self.place, self.time("wait")).ok()?;
        wire.set_read_timeout(Some(self.time("answer"))).ok()?;
        wire.write_all(asked.as_bytes()).ok()?;
        let mut reader = BufReader::new(wire);
        let (status, said) = head(&mut reader).ok()?;
        let body = net::body(&mut reader, &said)?;
        Some((status, said, String::from_utf8(body).ok()?))
    }

    /// What Phiora answers a path with. Nothing when it is not there,
    /// when it refuses, and when it takes longer than it is waited for.
    pub fn get(&self, path: &str) -> Option<String> {
        self.ask(Verb::Get, path, "").filter(|(status, ..)| status == self.word.get("fine")).map(|(.., body)| body)
    }

    /// An app's own document, as Phiora composes it from its records.
    pub fn app(&self, key: &str) -> Option<String> {
        self.get(&self.word.get("app").replace("{key}", key))
    }

    /// Be somebody to Phiora: with the session it gave before, or a new
    /// one, which it gives whoever asks who they are. `false` when it
    /// does not answer.
    pub fn enter(&mut self) -> bool {
        let Some((status, said, _)) = self.ask(Verb::Get, self.word.get("me"), "") else { return false };
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
        let taken = self.ask(Verb::Put, &self.word.get("state").replace("{name}", name), document);
        taken.is_some_and(|(status, ..)| [self.word.get("fine"), self.word.get("kept")].contains(&status.as_str()))
    }

    /// Ask for something to be done, and hear the answer: whether
    /// Phiora did it, and what it said. Nothing when it is not there.
    pub fn said(&self, path: &str, body: &str) -> Option<(bool, String)> {
        self.ask(Verb::Post, path, body).map(|(status, _, said)| (status == self.word.get("fine"), said))
    }

    /// Ask for something to be done: a body sent to a path. `false`
    /// when Phiora did not do it.
    pub fn post(&self, path: &str, body: &str) -> bool {
        self.ask(Verb::Post, path, body).is_some_and(|(status, ..)| status == self.word.get("fine"))
    }

    /// Make, change or take away a record (`/tree/item`): the kind of
    /// ask (`post`, `patch` or `delete`), its path and its body. Whether
    /// Phiora did it, and what it said; nothing when it is not there.
    pub fn record(&self, verb: Verb, path: &str, body: &str) -> Option<(bool, String)> {
        let done = [self.word.get("fine"), self.word.get("made")];
        self.ask(verb, path, body).map(|(status, _, said)| (done.contains(&status.as_str()), said))
    }
}