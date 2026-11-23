//! What is true of the person using Phiogy now, as the profile shows it
//! under their name: where the line is, how many rooms have a page on
//! it and how many people are in calls, how many hosts the site has
//! and where they were read from,
//! the look they chose and where that is kept, and which Phiogy this
//! is. It is asked once a frame.
//!
//! What each row is called is `data/text.tsv`'s: `fact.<name>`.

use appkit::service::PTable;

use crate::board::lock;
use crate::part::Desk;
use crate::site::Site;

/// Where a site was read from, as a person reads it: Phiora, the
/// app's own files, or the files because Phiora did not answer.
pub fn read(site: &Site, text: &PTable) -> String {
    match &site.asked {
        Some((at, true)) => text.fill("fact.phiora", "at", at),
        Some((at, false)) => text.fill("fact.unanswered", "at", at),
        None => text.get("fact.file").to_string(),
    }
}

/// The rows, a name and what it stands for.
pub fn fact(desk: &Desk) -> PTable {
    let text = &desk.text;
    let open = lock(&desk.board).open();
    let (room, call) = (open.len(), open.iter().map(|(_, call, _)| call).sum::<usize>());
    let count = |count: usize| text.fill("fact.count", "count", &count.to_string());
    let line = match desk.line.as_ref() {
        Ok(at) => at.clone(),
        Err(_) => text.get("fact.closed").to_string(),
    };
    let place = desk.choice.place().unwrap_or_else(|| text.get("fact.nowhere").to_string());
    let row = [
        ("fact.line", line),
        ("fact.room", count(room)),
        ("fact.call", count(call)),
        ("fact.venue", count(desk.site.venue.len())),
        ("fact.site", read(&desk.site, text)),
        ("fact.look", text.get(&format!("look.{}", desk.shade.name())).to_string()),
        ("fact.keep", place),
        ("fact.version", env!("CARGO_PKG_VERSION").to_string()),
    ];
    let line: String = row.iter().filter(|(_, value)| !value.is_empty()).map(|(name, value)| format!("{}\t{value}\n", text.get(name))).collect();
    PTable::read(&line)
}
