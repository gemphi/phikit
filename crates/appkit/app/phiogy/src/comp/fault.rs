//! A fault: why the line refuses. The rules name the fault; the status
//! it is answered with, the word a page reads it by and the sentence
//! for a person are `data/fault.tsv`.

use crate::data;

/// Why the line refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// The page gave no name a peer may have.
    Peer,
    /// The peer is someone else's.
    Owner,
    /// The room's line holds as many pages as it may.
    Watch,
    /// The one who speaks has no line open under that name.
    Line,
    /// The call holds as many as it may.
    Caller,
    /// The page that calls is not one that may.
    Origin,
    /// Nothing answers at that path.
    Route,
    /// The request is larger than the line reads.
    Large,
    /// What was posted is not JSON.
    Json,
    /// The line serves as many pages as it may.
    Wire,
    /// No app has that key.
    App,
    /// A document a person keeps may not have that name.
    Name,
    /// The person keeps no document of that name.
    Unset,
    /// Nothing was sent to keep.
    Empty,
}

impl Fault {
    /// What the table calls it.
    pub fn slug(self) -> &'static str {
        match self {
            Fault::Peer => "peer",
            Fault::Owner => "owner",
            Fault::Watch => "watch",
            Fault::Line => "line",
            Fault::Caller => "caller",
            Fault::Origin => "origin",
            Fault::Route => "route",
            Fault::Large => "large",
            Fault::Json => "json",
            Fault::Wire => "wire",
            Fault::App => "app",
            Fault::Name => "name",
            Fault::Unset => "unset",
            Fault::Empty => "empty",
        }
    }
}

/// What the line says for each fault.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Refusal {
    /// The fault, its status, its word and its sentence.
    row: Vec<(String, u16, String, String)>,
}

impl Refusal {
    /// The refusals of a table: a fault a line, then the status, the
    /// word and the sentence.
    pub fn read(text: &str) -> Refusal {
        let row = data::row(text).into_iter().map(|cell| {
            let word = |index: usize| cell.get(index).copied().unwrap_or_default().to_string();
            (word(0), word(1).parse().unwrap_or_default(), word(2), word(3))
        });
        Refusal { row: row.collect() }
    }

    /// The status, the word and the sentence of a fault; nothing for one
    /// the table does not have.
    pub fn of(&self, fault: Fault) -> (u16, &str, &str) {
        let found = self.row.iter().find(|(slug, ..)| slug == fault.slug());
        found.map_or((0, "", ""), |(_, status, word, line)| (*status, word.as_str(), line.as_str()))
    }
}
