//! What is true of the person using Phigen now, as the profile shows it
//! under their name: which model answers and where it runs, the look
//! they chose, how many chats they have and where those are kept, and
//! which Phigen this is. It is asked once a frame, so a model chosen in
//! the settings is the one the profile names the next time it is drawn.
//!
//! What each row is called is `data/text.tsv`'s: `fact.<name>`.

use appkit::service::PTable;

use crate::data;
use crate::part::Desk;

/// The rows, a name and what it stands for.
pub fn fact(desk: &Desk) -> PTable {
    let text = &desk.text;
    let history = desk.history.borrow();
    // The server is named only when a model on it answers.
    let (model, server) = match history.voice() {
        "" => (text.get("model.none").to_string(), String::new()),
        name => (name.to_string(), PTable::read(data::MODEL).get("host").to_string()),
    };
    let said = history.list().iter().filter(|chat| chat.title().is_some()).count();
    let place = desk.choice.place().unwrap_or_else(|| text.get("fact.nowhere").to_string());
    let row = [
        ("fact.model", model),
        ("fact.server", server),
        ("fact.look", text.get(&format!("look.{}", desk.shade.name())).to_string()),
        ("fact.chat", text.fill("fact.count", "count", &said.to_string())),
        ("fact.keep", place),
        ("fact.version", env!("CARGO_PKG_VERSION").to_string()),
    ];
    let line: String = row.iter().filter(|(_, value)| !value.is_empty()).map(|(name, value)| format!("{}\t{value}\n", text.get(name))).collect();
    PTable::read(&line)
}
