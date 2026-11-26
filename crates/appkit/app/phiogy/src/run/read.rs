//! What `phiogy --read` says: where its hosts and rooms were read
//! from, who the person is to Phiora, each host with its rooms, and
//! what was said in the room it is opened on.

use appkit::service::PTable;

use crate::part::Desk;

/// What was read, as lines to print: where from, each host with its
/// rooms, and what was said in the room it is opened on.
pub(super) fn read(desk: &Desk, text: &PTable) -> Vec<String> {
    let site = &desk.site;
    let from = text.fill("read.from", "at", &crate::fact::read(site, text));
    let made = desk.made();
    let venue = site.venue.iter().chain(made.iter()).flat_map(|venue| {
        let head = text.fill("read.venue", "label", &venue.label).replace("{count}", &venue.plate.len().to_string());
        let room = venue.plate.iter().map(|plate| text.fill("read.room", "id", &plate.id).replace("{label}", &plate.label));
        std::iter::once(head).chain(room).collect::<Vec<_>>()
    });
    let room = desk.room();
    // Each message, and under it the marks it holds (a like, and how many gave it).
    let said = desk.said().into_iter().flat_map(|one| {
        let line = text.fill("read.said", "room", &room).replace("{by}", &one.by).replace("{at}", &one.at).replace("{text}", &one.text);
        let marks = desk.mark(&one).into_iter().filter(|(_, count, _)| *count > 0).map(|(icon, count, _)| text.fill("read.mark", "icon", &icon).replace("{count}", &count.to_string()).replace("{text}", &one.text));
        std::iter::once(line).chain(marks).collect::<Vec<_>>()
    });
    let kept = &desk.kept;
    let me = Some(kept.me()).filter(|me| !me.is_null()).map(|me| text.fill("read.me", "name", &crate::part::who(&me, &|name| kept.word(name), text)));
    let session = Some(kept.session().len()).filter(|count| *count > 0 && !kept.guest()).map(|count| text.fill("read.session", "count", &count.to_string()));
    let me = me.into_iter().chain(session);
    std::iter::once(from).chain(me).chain(venue).chain(said).collect()
}
