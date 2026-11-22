//! The hosts: the page a person adds a host from, as the web's is
//! (`Hosts` in `puikit/app/phi-kit/src/hosts.tsx`). First, a host of
//! their own: a name, then how it starts, one of the catalogue's
//! templates; the hosts they made stand under it, each with the way to
//! remove it. Then every host there is to join: a field that searches
//! them, the first that match with their mark, how many people they
//! have, and joining or leaving. A host that is joined stands in the
//! rail; a press on its line opens its first room.

use std::cell::Cell;

use appkit::pui::control::{PControl, PField};
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};

use super::list::{Deed, Item, Lister, Seat, field, grouped};
use super::Desk;

/// How many hosts of the catalogue are listed at most, as on the web.
pub const MOST: &str = "host.most";

/// What a press on the page means.
#[derive(Debug, Clone, PartialEq)]
enum Act {
    /// Give the keys to the field of that number.
    Field(usize),
    /// Make a host from the template of that number.
    Make(usize),
    Remove(String),
    Join(String, bool),
    Open(String),
}

/// The hosts.
pub struct Venue {
    frame: PRect,
    desk: Desk,
    list: Lister<Act>,
    /// The name of a host to make, and what the hosts are searched by;
    /// and which of the two has the keys.
    name: PField,
    find: PField,
    aim: Cell<usize>,
}

impl Venue {
    pub fn new(desk: Desk) -> Venue {
        Venue { frame: PRect::default(), desk, list: Lister::default(), name: PField::new(PRect::default()), find: PField::new(PRect::default()), aim: Cell::new(1) }
    }

    /// What the page holds, from its top down.
    fn item(&self) -> Vec<Item<Act>> {
        let (text, site) = (&self.desk.text, &self.desk.site);
        let word = |name: &str| text.get(name).to_string();
        let named = !self.name.value().trim().is_empty();
        let mut item = vec![Item::Head(word("host.create")), Item::Note(word("host.lead")), Item::Field(0, word("host.name"), Act::Field(0))];
        // A template can be pressed once the host has a name.
        for (index, template) in site.extra.template.iter().enumerate() {
            let room = template.room.iter().map(|(label, ..)| label.as_str()).collect::<Vec<_>>().join(&format!(" {} ", text.get("between")));
            item.push(Item::Line(Seat::Icon(self.desk.icon(&template.icon).to_string()), template.label.clone(), room, Some(Act::Make(index)).filter(|_| named), Vec::new()));
        }
        for made in self.desk.made() {
            let first = made.plate.first().map(|plate| Act::Open(plate.id.clone()));
            item.push(Item::Line(Seat::Icon(self.desk.icon(&made.icon).to_string()), made.label.clone(), String::new(), first, vec![Deed::word(text.get("host.remove"), false, Act::Remove(made.id.clone()))]));
        }
        item.extend([Item::Head(word("host.join")), Item::Field(1, word("host.find"), Act::Field(1))]);
        let wanted = self.find.value().trim().to_lowercase();
        let all = &site.venue[site.extra.nav.min(site.venue.len())..];
        let most = self.desk.size(MOST) as usize;
        let found: Vec<_> = all.iter().filter(|venue| wanted.is_empty() || venue.label.to_lowercase().contains(&wanted)).take(most).collect();
        for venue in &found {
            let inside = self.desk.joined(venue);
            let seat = venue.mark.clone().map_or_else(|| Seat::Icon(self.desk.icon(&venue.icon).to_string()), Seat::Face);
            let deed = match inside {
                true => Deed::word(text.get("host.leave"), true, Act::Join(venue.id.clone(), false)),
                false => Deed::word(text.get("host.enter"), false, Act::Join(venue.id.clone(), true)),
            };
            let first = venue.plate.first().map(|plate| Act::Open(plate.id.clone()));
            item.push(Item::Line(seat, venue.label.clone(), text.fill("host.members", "n", &grouped(venue.count)), first, vec![deed]));
        }
        if found.is_empty() {
            item.push(Item::Note(word("host.none")));
        }
        item.push(Item::Note(text.fill("host.count", "n", &all.len().to_string())));
        item
    }
}

impl PDraw for Venue {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.list.draw(paint, origin, self.frame, &self.desk, &self.item(), &[&self.name, &self.find]);
    }
}

impl PView for Venue {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
        self.name.set_frame(field(&self.desk, frame.size.width));
        self.find.set_frame(field(&self.desk, frame.size.width));
    }

    fn tick(&mut self) -> bool {
        (self.name.focused, self.find.focused) = (self.aim.get() == 0, self.aim.get() == 1);
        false
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        self.tick();
        match *event {
            PEvent::KeyDown(_) => match self.aim.get() {
                0 => self.name.respond(event),
                _ => self.find.respond(event),
            },
            PEvent::Scroll(_, by) => self.list.wheel(by.y),
            PEvent::PointerDown(at) => {
                let Some(act) = self.list.press(at) else { return false };
                match act {
                    Act::Field(index) => self.aim.set(index),
                    Act::Make(index) => {
                        let made = self.desk.site.extra.template.get(index).and_then(|template| self.desk.create(&self.name.value(), template));
                        self.name.store(String::new());
                        made.into_iter().for_each(|room| _ = self.desk.show(&room));
                    }
                    Act::Remove(id) => self.desk.unmake(&id),
                    Act::Join(id, on) => self.desk.enter(&id, on),
                    Act::Open(room) => _ = self.desk.show(&room),
                }
                true
            }
            _ => false,
        }
    }
}
