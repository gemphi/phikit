//! The words the line and a page say to each other, as they are spelled
//! on the wire: `data/wire.tsv`. The browser's side reads the same ones
//! (`puikit/app/phi-kit/src/call.ts`), so a word changes there and in
//! the table, never here.

use appkit::service::PTable;

use crate::data;

/// The words on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Wire {
    pub roster: String,
    pub ice: String,
    pub peer: String,
    pub name: String,
    pub handle: String,
    pub from: String,
    pub signal: String,
    pub join: String,
    pub send: String,
    pub to: String,
    pub error: String,
    pub message: String,
    /// How an event of the stream begins, and the line that is a beat.
    pub event: String,
    pub beat: String,
}

impl Wire {
    /// The words of a table: what the code calls each, and its spelling.
    pub fn read(text: &str) -> Wire {
        let table = PTable::read(text);
        let word = |name: &str| table.get(name).to_string();
        Wire { roster: word("roster"), ice: word("ice"), peer: word("peer"), name: word("name"), handle: word("handle"), from: word("from"), signal: word("signal"), join: word("join"), send: word("send"), to: word("to"), error: word("error"), message: word("message"), event: word("event"), beat: word("beat") }
    }

    /// The words Phiogy ships with.
    pub fn own() -> Wire {
        Wire::read(data::WIRE)
    }
}
