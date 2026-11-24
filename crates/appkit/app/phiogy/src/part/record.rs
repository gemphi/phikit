//! A record's page, as Phiegg shows one: a world, a club or a show that
//! search found (`now.record`). Its picture across the page; its name,
//! with the way to its host where it has one; its line; the facts of
//! its kind (`data/record.tsv`), each on a ground of its own; the records
//! it is related to (its world, its club, what stands in it) as cards,
//! each opening its own page; and its host's rooms as cards, each going
//! there. Nothing of a kind is written here: the fields are data.

use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::window::PDeed;
use serde_json::Value;

use super::list::{Card, Deed, Item, Lister, Seat};
use super::search::Found;
use super::Desk;
use crate::data;

/// The modal this is, as `data/modal.tsv` names it.
pub const RECORD: &str = "record";

/// What a press on it means.
#[derive(Debug, Clone, PartialEq)]
enum Act {
    /// Go to the room of that id.
    Go(String),
    /// Open the page of the record of that kind and id.
    Open(String, String),
}

/// A record's page.
pub struct Record {
    frame: PRect,
    desk: Desk,
    list: Lister<Act>,
}

impl Record {
    pub fn new(desk: Desk) -> Record {
        Record { frame: PRect::default(), desk, list: Lister::default() }
    }

    /// A fact of a record: its field as a person reads it.
    fn fact(value: &Value) -> Option<String> {
        match value {
            Value::String(text) if !text.is_empty() => Some(text.clone()),
            Value::Number(number) => Some(number.as_u64().map_or_else(|| number.to_string(), super::tool::grouped)),
            _ => None,
        }
    }

    fn item(&self, one: &Found) -> Vec<Item<Act>> {
        let (text, site) = (&self.desk.text, &self.desk.site);
        let mut item = Vec::new();
        if !one.picture.is_empty() {
            item.push(Item::Hero(one.picture.clone()));
        }
        // Its host, by the field the search table names.
        let field = data::row(data::SEARCH).into_iter().find(|row| row.first() == Some(&"host")).and_then(|row| row.get(1).map(|cell| cell.to_string())).unwrap_or_default();
        let host = one.row.get(&field).and_then(Value::as_str).and_then(|id| site.venue.iter().find(|venue| venue.id == id));
        let first = host.and_then(|venue| venue.plate.first()).map(|plate| plate.id.clone());
        item.push(Item::Top(one.name.clone(), first.iter().map(|room| Deed::word(text.get("record.host"), false, Act::Go(room.clone()))).collect()));
        if !one.line.is_empty() {
            item.push(Item::Note(one.line.clone()));
        }
        // The facts of its kind.
        for row in data::row(data::RECORD).into_iter().filter(|row| row.first() == Some(&one.kind.as_str()) && row.len() >= 3) {
            if let Some(value) = one.row.get(row[1]).and_then(Record::fact) {
                item.push(Item::Sum(text.get(row[2]).to_string(), value));
            }
        }
        // What it is related to, each a card that opens its page.
        let near = self.desk.book().related(one);
        if !near.is_empty() {
            item.push(Item::Top(text.get("record.near").to_string(), Vec::new()));
            item.push(Item::Grid(near.iter().map(|other| Card {
                seat: match other.picture.is_empty() {
                    true => Seat::Icon(self.desk.icon("globe").to_string()),
                    false => Seat::Picture(other.picture.clone()),
                },
                title: other.name.clone(),
                sub: text.get(&format!("search.{}", other.kind)).to_string(),
                act: Some(Act::Open(other.kind.clone(), other.id.clone())),
            }).collect()));
        }
        // Its host's rooms, each a card that goes there.
        if let Some(venue) = host {
            item.push(Item::Top(text.get("record.rooms").to_string(), Vec::new()));
            item.push(Item::Grid(venue.plate.iter().map(|plate| Card {
                seat: venue.mark.clone().map_or_else(|| Seat::Icon(self.desk.icon(&plate.icon).to_string()), Seat::Face),
                title: plate.label.clone(),
                sub: venue.label.clone(),
                act: Some(Act::Go(plate.id.clone())),
            }).collect()));
        }
        item
    }
}

impl PDraw for Record {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let Some(one) = self.desk.now.record.borrow().clone() else { return };
        self.list.draw(paint, origin, self.frame, &self.desk, &self.item(&one), &[]);
    }
}

impl PView for Record {
    fn hand(&self) -> Vec<PRect> {
        self.list.hands()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match *event {
            PEvent::Scroll(_, by) => self.list.wheel(by.y),
            PEvent::PointerDown(at) => {
                let Some(act) = self.list.press(at) else { return false };
                match act {
                    // Going to a room shuts the page.
                    Act::Go(id) => {
                        if self.desk.show(&id) {
                            self.desk.wish.borrow_mut().push(PDeed::Close);
                        }
                    }
                    Act::Open(kind, id) => *self.desk.now.record.borrow_mut() = self.desk.book().record(&kind, &id).cloned(),
                }
                true
            }
            _ => false,
        }
    }
}
