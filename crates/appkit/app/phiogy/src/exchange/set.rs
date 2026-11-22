//! What the exchange is set up with, all of it read from `data/`: where
//! it listens, the first part of a room's path, the cookie a page is
//! known by, what a guest is called, which pages may call from a
//! browser, the lines at the head of each answer, what each status is
//! called, what it says when it refuses, the words on the wire and the
//! ICE servers. Two of these the system may give instead: the servers,
//! and the pages that may call.

use appkit::service::PTable;

use super::told::Told;
use crate::comp::{Ice, Refusal, Wire};
use crate::data;

/// The kinds of line `data/head.tsv` has.
pub const STREAM: &str = "stream";
pub const JSON: &str = "json";
pub const CORS: &str = "cors";
pub const ASK: &str = "ask";
pub const COOKIE: &str = "cookie";
pub const LENGTH: &str = "length";

/// The exchange's settings.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Set {
    pub host: String,
    pub port: u16,
    /// The first part of a room's path.
    pub path: String,
    /// The protocol an answer names itself by.
    pub speak: String,
    pub cookie: String,
    /// What a guest is called, and how their id begins.
    pub guest: String,
    pub id: String,
    /// The pages that may call, and whether each must match whole: one
    /// the system gave does, one of the table's may have any port.
    origin: Vec<(String, bool)>,
    head: Vec<(String, String)>,
    status: PTable,
    pub refusal: Refusal,
    pub wire: Wire,
    pub ice: Ice,
    /// Who is believed when they say who a page belongs to.
    pub told: Told,
}

impl Set {
    /// The settings Phiogy ships with, and what the system gives.
    /// `change` is rows that stand before the table's own, such as
    /// another port.
    pub fn own(change: &str) -> Set {
        let line = PTable::read(&format!("{change}\n{}", data::LINE));
        let given = |name: &str| std::env::var(line.get(name)).ok().filter(|value| !value.trim().is_empty());
        Set::read(&line, given("ice").as_deref(), given("allow").as_deref()).trust(given("trust").as_deref())
    }

    /// The settings of the line's table, with a JSON list of servers
    /// and a list of pages that may call, where either is given.
    pub fn read(line: &PTable, ice: Option<&str>, allow: Option<&str>) -> Set {
        let word = |name: &str| line.get(name).to_string();
        let listed = line.list().iter().filter(|(name, _)| name == "origin").map(|(_, value)| (value.clone(), false));
        let origin = match allow {
            Some(list) => list.split(',').map(|one| (one.trim().to_string(), true)).filter(|(one, _)| !one.is_empty()).collect(),
            None => listed.collect(),
        };
        let head = data::row(data::HEAD).into_iter().filter_map(|cell| Some((cell.first()?.to_string(), cell.get(1)?.to_string())));
        Set { host: word("host"), port: word("port").parse().unwrap_or_default(), path: word("path"), speak: word("speak"), cookie: word("cookie"), guest: word("guest"), id: word("id"), origin, head: head.collect(), status: PTable::read(data::STATUS), refusal: Refusal::read(data::FAULT), wire: Wire::own(), ice: Ice::read(data::ICE, ice), told: Told::read(line) }
    }

    /// The same settings, believing whoever carries that word about who
    /// a page belongs to; with no word nobody is believed.
    pub fn trust(mut self, word: Option<&str>) -> Set {
        self.told = self.told.believing(word);
        self
    }

    /// The lines of one kind at the head of an answer.
    pub fn head<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a str> {
        self.head.iter().filter(move |(known, _)| known == kind).map(|(_, line)| line.as_str())
    }

    /// What a status is called.
    pub fn status(&self, code: u16) -> &str {
        self.status.get(&code.to_string())
    }

    /// Whether a page of that origin may call from a browser.
    pub fn allowed(&self, origin: &str) -> bool {
        let port = |rest: &str| rest.strip_prefix(':').is_some_and(|port| !port.is_empty() && port.chars().all(|digit| digit.is_ascii_digit()));
        let fits = |(known, whole): &(String, bool)| match (origin.strip_prefix(known.as_str()), whole) {
            (Some(""), _) => true,
            (Some(rest), false) => port(rest),
            _ => false,
        };
        !origin.is_empty() && self.origin.iter().any(fits)
    }
}
