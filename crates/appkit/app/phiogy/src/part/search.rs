//! Search, as Phiegg's is: the window in the middle that the title's
//! field opens. A field; tabs of what it finds (`data/search.tsv`): all
//! of it, the rooms of every host the person is in, people, hosts, and
//! Phiegg's own worlds, clubs and shows, the very records Phiegg reads
//! (`search/book.rs`); and before anything is sought, what is sought
//! most often, a press on which seeks it. A press on a room goes to it,
//! on a host to its first room, on somebody to a conversation with
//! them. It is a list page as Friends is (`list.rs`), so it has no
//! drawing of its own.

mod book;

use std::cell::Cell;

use appkit::pui::control::{PControl, PField};
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::window::PDeed;

use super::Desk;
use super::friend::MOST;
use super::list::{Deed, Item, Lister, Seat, field};
use crate::data;
use book::Book;

/// The modal this is, as `data/modal.tsv` names it.
pub const SEARCH: &str = "search";
/// What a tab finds, past the room, the people and the hosts: Phiora's
/// records of a kind, after this; and every other tab at once.
const TREE: &str = "tree:";
const ALL: &str = "all";

/// What a press on it means.
#[derive(Debug, Clone, PartialEq)]
enum Act {
    Field,
    /// Go to the room of that id.
    Go(String),
    /// Write to the person of that id.
    Write(String),
    /// Choose the tab of that number.
    Tab(usize),
    /// Seek these words.
    Seek(String),
}

/// A tab: the word on it, what it finds, the icon its finds carry.
struct Tab {
    word: String,
    finds: String,
    icon: String,
}

/// The Search window.
pub struct Search {
    frame: PRect,
    desk: Desk,
    list: Lister<Act>,
    find: PField,
    tab: Cell<usize>,
    tabs: Vec<Tab>,
    book: Book,
}

impl Search {
    pub fn new(desk: Desk) -> Search {
        let mut find = PField::new(PRect::default());
        find.focused = true;
        let tabs: Vec<Tab> = data::row(data::SEARCH).into_iter().filter(|row| row.first() == Some(&"tab") && row.len() >= 5)
            .map(|row| Tab { word: row[2].to_string(), finds: row[3].to_string(), icon: row[4].trim_matches('-').to_string() }).collect();
        let kinds: Vec<&str> = tabs.iter().filter_map(|tab| tab.finds.strip_prefix(TREE)).collect();
        let book = Book::read(&desk.kept, &kinds);
        Search { frame: PRect::default(), desk, list: Lister::default(), find, tab: Cell::default(), tabs, book }
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

    /// What a tab finds.
    fn found(&self, tab: &Tab, wanted: &str, most: usize) -> Vec<Item<Act>> {
        let icon = Seat::Icon(self.desk.icon(&tab.icon).to_string());
        let holds = |name: &str| wanted.is_empty() || name.to_lowercase().contains(wanted);
        match tab.finds.as_str() {
            "room" => self.rooms(wanted, most),
            "people" => self.people(most),
            // A host: its members, and a press on it goes to its first room.
            "host" => self.desk.site.venue.iter().filter(|venue| holds(&venue.label) && !venue.plate.is_empty()).take(most)
                .map(|venue| Item::Line(venue.mark.clone().map_or_else(|| icon.clone(), Seat::Face), venue.label.clone(), super::tool::grouped(venue.count), venue.plate.first().map(|plate| Act::Go(plate.id.clone())), Vec::new())).collect(),
            finds => {
                let kind = finds.strip_prefix(TREE).unwrap_or(finds);
                let book = self.book.kind.iter().find(|(name, _)| name == kind).map(|(_, found)| found.as_slice()).unwrap_or_default();
                book.iter().filter(|found| holds(&found.name)).take(most).map(|found| Item::Line(icon.clone(), found.name.clone(), found.line.clone(), None, Vec::new())).collect()
            }
        }
    }

    fn item(&self) -> Vec<Item<Act>> {
        let text = &self.desk.text;
        let word = |name: &str| text.get(name).to_string();
        let (wanted, most) = (self.find.value().trim().to_lowercase(), self.desk.size(MOST) as usize);
        let chosen = self.tab.get().min(self.tabs.len().saturating_sub(1));
        // The window says what it is in its own head: here the field is
        // first, its tabs under it.
        let tabs = self.tabs.iter().enumerate().map(|(index, tab)| Deed::word(text.get(&tab.word), index == chosen, Act::Tab(index))).collect();
        let mut item = vec![Item::Field(0, word("search.hint"), Act::Field), Item::Tabs(tabs)];
        // Before anything is sought: what is sought most often.
        if wanted.is_empty() && !self.book.popular.is_empty() {
            item.push(Item::Head(word("search.popular")));
            item.push(Item::Tabs(self.book.popular.iter().map(|seek| Deed::word(seek, false, Act::Seek(seek.clone()))).collect()));
        }
        let shown: Vec<&Tab> = match self.tabs.get(chosen) {
            Some(tab) if tab.finds == ALL => self.tabs.iter().filter(|tab| tab.finds != ALL).collect(),
            Some(tab) => vec![tab],
            None => Vec::new(),
        };
        let mut any = false;
        for tab in shown {
            let list = self.found(tab, &wanted, most);
            if !list.is_empty() {
                any = true;
                item.push(Item::Sum(text.get(&tab.word).to_string(), list.len().to_string()));
                item.extend(list);
            }
        }
        if !any {
            item.push(Item::Note(word("search.none")));
        }
        item
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
                    Act::Tab(index) => {
                        self.tab.set(index);
                        false
                    }
                    Act::Seek(words) => {
                        self.find.store(words);
                        false
                    }
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
