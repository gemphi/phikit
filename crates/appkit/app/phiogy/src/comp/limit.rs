//! The limits: how many a call holds, how many pages a room's line, how
//! often a quiet line is told a beat and how long a page may take none,
//! and what a name on the line may be. Every number is `data/limit.tsv`
//! and every name's rule `data/name.tsv`; none is here.

use appkit::service::PTable;

use crate::data;

/// The kinds of name `data/name.tsv` has a rule for.
pub const APP: &str = "app";
pub const ROOM: &str = "room";
pub const PEER: &str = "peer";

/// What one kind of name may be: its fewest and its most characters, and
/// the marks it may have besides letters, digits and the underscore.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Span {
    kind: String,
    least: usize,
    most: usize,
    mark: String,
}

/// The limits of a call and of its line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Limit {
    /// How many are in one room's call.
    pub caller: usize,
    /// How many pages may hold one room's line.
    pub watch: usize,
    /// How often a line is told a beat, and how long a page that took
    /// none is kept, in milliseconds.
    pub beat: u64,
    pub gone: u64,
    /// How many signals one post may pass on.
    pub send: usize,
    /// How many bytes a request may be, how many pages the line serves
    /// at once, and how many guests it remembers.
    pub body: usize,
    pub wire: usize,
    pub session: usize,
    /// How often the line looks whether it was closed, in milliseconds.
    pub tick: u64,
    span: Vec<Span>,
}

impl Limit {
    /// The limits two tables give: a name and a number a line; and a
    /// kind of name, its fewest and most characters and its marks.
    pub fn read(limit: &str, name: &str) -> Limit {
        let table = PTable::read(limit);
        let count = |name: &str| table.get(name).parse::<usize>().unwrap_or_default();
        let span = data::row(name).into_iter().map(|cell| {
            let word = |index: usize| cell.get(index).copied().unwrap_or_default();
            Span { kind: word(0).to_string(), least: word(1).parse().unwrap_or_default(), most: word(2).parse().unwrap_or_default(), mark: word(3).to_string() }
        });
        Limit { caller: count("caller"), watch: count("watch"), beat: count("beat") as u64, gone: count("gone") as u64, send: count("send"), body: count("body"), wire: count("wire"), session: count("session"), tick: count("tick") as u64, span: span.collect() }
    }

    /// The limits Phiogy ships with.
    pub fn own() -> Limit {
        Limit::read(data::LIMIT, data::NAME)
    }

    /// Whether a text may be a name of that kind. A kind with no rule
    /// lets nothing through.
    pub fn fits(&self, kind: &str, text: &str) -> bool {
        let Some(span) = self.span.iter().find(|span| span.kind == kind) else { return false };
        let count = text.chars().count();
        let known = |letter: char| letter.is_ascii_alphanumeric() || letter == '_' || span.mark.contains(letter);
        count >= span.least && count <= span.most && text.chars().all(known)
    }
}
