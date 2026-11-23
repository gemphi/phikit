//! Search: the window in the middle that the title's field opens. A
//! field, and under it everything it finds: the rooms of every host
//! the person is in, and people. A press on a room goes to it; a press
//! on somebody begins or goes on with a conversation with them. It is
//! a list page as Friends is (`list.rs`), so it has no drawing of its
//! own.

use appkit::pui::control::{PControl, PField};
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::window::PDeed;

use super::friend::MOST;
use super::list::{Item, Lister, Seat, field};
use super::Desk;

/// The modal this is, as `data/modal.tsv` names it.
pub const SEARCH: &str = "search";

/// What a press on it means.
#[derive(Debug, Clone, PartialEq)]
enum Act {
    Field,
    /// Go to the room of that id.
    Go(String),
    /// Write to the person of that id.
    Write(String),
}

/// The Search window.
pub struct Search {
    frame: PRect,
    desk: Desk,
    list: Lister<Act>,
    find: PField,
}

impl Search {
    pub fn new(desk: Desk) -> Search {
        let mut find = PField::new(PRect::default());
        find.focused = true;
        Search { frame: PRect::default(), desk, list: Lister::default(), find }
    }

    /// The rooms whose name holds what is sought, each under its host.
    fn rooms(&self, wanted: &str, most: usize) -> Vec<Item<Act>> {
        let made = self.desk.made();
        let host = self.desk.site.venue.iter().chain(&made);
        let found = host.flat_map(|venue| venue.plate.iter().map(move |plate| (venue, plate)));
        let found = found.filter(|(_, plate)| wanted.is_empty() || plate.label.to_lowercase().contains(wanted)).take(most);
        found.map(|(venue, plate)| Item::Line(Seat::Icon(self.desk.icon(&plate.icon).to_string()), plate.label.clone(), venue.label.clone(), Some(Act::Go(plate.id.clone())), Vec::new())).collect()
    }

    /// The people it finds, each with their handle.
    fn people(&self, most: usize) -> Vec<Item<Act>> {
        let seat = |id: &str| self.desk.site.face(id).cloned().map_or_else(|| Seat::Icon(self.desk.icon(self.desk.page.face()).to_string()), Seat::Face);
        let found = self.desk.seek(&self.find.value(), most);
        found.iter().map(|one| Item::Line(seat(&one.id), one.name.clone(), Some(format!("@{}", one.handle)).filter(|_| !one.handle.is_empty()).unwrap_or_default(), Some(Act::Write(one.id.clone())), Vec::new())).collect()
    }

    fn item(&self) -> Vec<Item<Act>> {
        let text = &self.desk.text;
        let word = |name: &str| text.get(name).to_string();
        let (wanted, most) = (self.find.value().trim().to_lowercase(), self.desk.size(MOST) as usize);
        let (room, people) = (self.rooms(&wanted, most), self.people(most));
        let none = Some(Item::Note(word("search.none"))).filter(|_| room.is_empty() && people.is_empty());
        let under = |name: &str, list: Vec<Item<Act>>| Some(Item::Sum(word(name), list.len().to_string())).filter(|_| !list.is_empty()).into_iter().chain(list);
        // The window says what it is in its own head: here the field is first.
        let head = [Item::Field(0, word("search.hint"), Act::Field)];
        head.into_iter().chain(none).chain(under("search.room", room)).chain(under("search.people", people)).collect()
    }
}

impl PDraw for Search {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.list.draw(paint, origin, self.frame, &self.desk, &self.item(), &[&self.find]);
    }
}

impl PView for Search {
    fn hand(&self) -> Vec<PRect> {
        self.list.hands()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
        self.find.set_frame(field(&self.desk, frame.size.width));
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match *event {
            PEvent::KeyDown(_) => self.find.respond(event),
            PEvent::Scroll(_, by) => self.list.wheel(by.y),
            PEvent::PointerDown(at) => {
                let Some(act) = self.list.press(at) else { return false };
                // Going somewhere shuts the window it was found in.
                let went = match act {
                    Act::Field => false,
                    Act::Go(id) => self.desk.show(&id),
                    Act::Write(id) => self.desk.show(&self.desk.direct_room(&id)),
                };
                went.then(|| self.desk.wish.borrow_mut().push(PDeed::Close));
                true
            }
            _ => false,
        }
    }
}
