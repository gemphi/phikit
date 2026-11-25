//! Search, as Phiber's palette finds (phiber: modal/command/find): the
//! window in the middle that the title's field opens. A field; tabs of
//! what it finds along its top (`data/search.tsv`), each narrowing it to
//! one kind; and what it finds as cards in a grid, never a long list:
//! under All a few of each kind, with the way to all of them, and in a
//! kind's own tab as many as the theme says. Before anything is sought,
//! what is sought most often, a press on which seeks it. Every word
//! sought must begin a word of a thing's name or line, the best first
//! (`search/rank.rs`). A press on a room goes to it, on a host to its
//! first room, on somebody to a conversation with them, on a world, a
//! club or a show to its page (`record.rs`).

mod book;
pub mod rank;

pub use book::{Book, Found, said, values};

use std::cell::Cell;

use appkit::pui::control::{PControl, PField};
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::window::PDeed;

use super::Desk;
use super::friend::MOST;
use super::list::{Card, Deed, Item, Lister, Seat, field};
use crate::data;
use rank::sift;

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
    /// Open the page of the record of that kind and id.
    Record(String, String),
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
}

impl Search {
    pub fn new(desk: Desk) -> Search {
        let mut find = PField::new(PRect::default());
        find.focused = true;
        let tabs: Vec<Tab> = data::row(data::SEARCH).into_iter().filter(|row| row.first() == Some(&"tab") && row.len() >= 5)
            .map(|row| Tab { word: row[2].to_string(), finds: row[3].to_string(), icon: row[4].trim_matches('-').to_string() }).collect();
        Search { frame: PRect::default(), desk, list: Lister::default(), find, tab: Cell::default(), tabs }
    }

    /// What a tab finds, ranked, as cards: no more than `most`, and how
    /// many it found in all.
    fn found(&self, tab: &Tab, sought: &str, most: usize) -> (usize, Vec<Card<Act>>) {
        let icon = Seat::Icon(self.desk.icon(&tab.icon).to_string());
        let card = |seat: Seat, title: &str, sub: &str, act: Option<Act>| Card { seat, title: title.to_string(), sub: sub.to_string(), act };
        let all = usize::MAX;
        match tab.finds.as_str() {
            // The rooms of every host, each with its host's mark.
            "room" => {
                let made = self.desk.made();
                let host = self.desk.site.venue.iter().chain(&made);
                let rooms: Vec<_> = host.flat_map(|venue| venue.plate.iter().map(move |plate| (venue, plate))).collect();
                let found = sift(rooms, sought, all, |(venue, plate)| (plate.label.clone(), venue.label.clone()));
                // A direct room wears the face of who it is with; any other its host's mark.
                let seat = |venue: &crate::site::Venue, plate: &crate::site::Plate| self.desk.site.face(&plate.with).cloned().or_else(|| venue.mark.clone()).map_or_else(|| Seat::Icon(self.desk.icon(&plate.icon).to_string()), Seat::Face);
                let cards = found.iter().take(most).map(|(venue, plate)| card(seat(venue, plate), &plate.label, &venue.label, Some(Act::Go(plate.id.clone())))).collect();
                (found.len(), cards)
            }
            "people" => {
                let others = self.desk.site.member.iter().filter(|one| one.id != self.desk.site.me);
                let found = sift(others, sought, all, |one| (one.name.clone(), one.handle.clone()));
                let seat = |id: &str| self.desk.site.face(id).cloned().map_or_else(|| Seat::Icon(self.desk.icon(self.desk.page.face()).to_string()), Seat::Face);
                let cards = found.iter().take(most).map(|one| card(seat(&one.id), &one.name, &format!("@{}", one.handle), Some(Act::Write(one.id.clone())))).collect();
                (found.len(), cards)
            }
            // A host: its mark, its members; a press goes to its first room.
            "host" => {
                let hosts = self.desk.site.venue.iter().filter(|venue| !venue.plate.is_empty());
                let found = sift(hosts, sought, all, |venue| (venue.label.clone(), String::new()));
                let cards = found.iter().take(most).map(|venue| card(venue.mark.clone().map_or_else(|| icon.clone(), Seat::Face), &venue.label, &super::tool::grouped(venue.count), venue.plate.first().map(|plate| Act::Go(plate.id.clone())))).collect();
                (found.len(), cards)
            }
            // Phiegg's own records, each its picture or its world's.
            finds => {
                let kind = finds.strip_prefix(TREE).unwrap_or(finds);
                let book = self.desk.book().kind.iter().find(|(name, _)| name == kind).map(|(_, found)| found.as_slice()).unwrap_or_default();
                let found = sift(book, sought, all, |found| (found.name.clone(), found.line.clone()));
                let seat = |picture: &str| match picture.is_empty() {
                    true => icon.clone(),
                    false => Seat::Picture(picture.to_string()),
                };
                let cards = found.iter().take(most).map(|found| card(seat(&found.picture), &found.name, &found.line, Some(Act::Record(found.kind.clone(), found.id.clone())))).collect();
                (found.len(), cards)
            }
        }
    }

    fn item(&self) -> Vec<Item<Act>> {
        let text = &self.desk.text;
        let word = |name: &str| text.get(name).to_string();
        let sought = self.find.value();
        let chosen = self.tab.get().min(self.tabs.len().saturating_sub(1));
        // The field first, its tabs under it.
        let tabs = self.tabs.iter().enumerate().map(|(index, tab)| Deed::word(text.get(&tab.word), index == chosen, Act::Tab(index))).collect();
        let mut item = vec![Item::Field(0, word("search.hint"), Act::Field), Item::Tabs(tabs)];
        // Before anything is sought: what is sought most often.
        let popular = &self.desk.book().popular;
        if sought.trim().is_empty() && !popular.is_empty() {
            item.push(Item::Head(word("search.popular")));
            item.push(Item::Tabs(popular.iter().map(|seek| Deed::word(seek, false, Act::Seek(seek.clone()))).collect()));
        }
        let few = data::row(data::SEARCH).into_iter().find(|row| row.first() == Some(&"all")).and_then(|row| row.get(1)?.parse().ok()).unwrap_or(8);
        let shown: Vec<(usize, &Tab)> = match self.tabs.get(chosen) {
            // Under All, a few of each kind, with the way to all of them.
            Some(tab) if tab.finds == ALL => self.tabs.iter().enumerate().filter(|(_, tab)| tab.finds != ALL).collect(),
            Some(tab) => vec![(chosen, tab)],
            None => Vec::new(),
        };
        let all = self.tabs.get(chosen).is_some_and(|tab| tab.finds == ALL);
        let most = match all {
            true => few,
            false => self.desk.size(MOST) as usize,
        };
        let mut any = false;
        for (index, tab) in shown {
            let (count, cards) = self.found(tab, &sought, most);
            if cards.is_empty() {
                continue;
            }
            any = true;
            let head = format!("{} {}", text.get(&tab.word), super::tool::grouped(count as u64));
            let more = (all && count > cards.len()).then(|| Deed::word(text.get("search.every"), false, Act::Tab(index)));
            item.push(Item::Top(head, more.into_iter().collect()));
            item.push(Item::Grid(cards));
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
                    // A record's page opens in the place of this window.
                    Act::Record(kind, id) => {
                        *self.desk.now.record.borrow_mut() = self.desk.book().record(&kind, &id).cloned();
                        self.desk.wish.borrow_mut().push(PDeed::Open(super::record::RECORD.to_string()));
                        false
                    }
                };
                went.then(|| self.desk.wish.borrow_mut().push(PDeed::Close));
                true
            }
            _ => false,
        }
    }
}
