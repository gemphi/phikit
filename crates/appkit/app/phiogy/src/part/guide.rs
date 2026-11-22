//! The guide: the places a person joined, each a heading with its rooms
//! under it, the open room marked. It is the part of the guide host,
//! which appkit shows in a group of the bar's menu (`data/menu.tsv`):
//! beside the bar at a desk, drawn out over the room in a hand. A room
//! with somebody on its line says how many. A press on a room opens it;
//! the word `start`, from a row of the menu, opens the room a person
//! begins in; and a look chosen in the menu or the profile is kept.

use std::cell::Cell;

use appkit::pui::core::{PDraw, PEvent, PFit, PLight, PPaint, PPoint, PRect, PView, light};
use appkit::pui::layout::PScroll;

use super::{DIM, Desk, GAP, ICON, INK, LINE, RADIUS, ROW, Rung, fit};
use crate::board::lock;

/// The word a row of the menu tells to open the room a person begins
/// in; and the word before a look's name when one was chosen.
pub const START: &str = "start";
pub const LOOK: &str = "look ";

/// A row of the guide: a place's heading, or one of its rooms.
enum Row {
    Head(String),
    /// A heading under a place, over some of its rooms: Text, Voice.
    Under(String),
    Room(Rung),
}

/// The places a person joined, and their rooms.
pub struct Guide {
    frame: PRect,
    desk: Desk,
    scroll: Cell<PScroll>,
}

impl Guide {
    pub fn new(desk: Desk) -> Guide {
        Guide { frame: PRect::default(), desk, scroll: Cell::default() }
    }

    /// Every row, in order.
    fn row(&self) -> Vec<Row> {
        let mut list = Vec::new();
        for (label, rung) in self.desk.rail() {
            list.push(Row::Head(label));
            let mut under = String::new();
            for rung in rung {
                if rung.head != under {
                    under = rung.head.clone();
                    list.push(Row::Under(under.clone()));
                }
                list.push(Row::Room(rung));
            }
        }
        list
    }
}

impl PDraw for Guide {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, row, line, icon, width) = (paint.size(GAP), paint.size(ROW), paint.size(LINE), paint.size(ICON), self.frame.size.width);
        let (ink, dim, radius) = (paint.color(INK), paint.color(DIM), paint.size(RADIUS));
        let down = (row - line) / 2.0;
        let list = self.row();
        if list.is_empty() {
            return paint.text(origin.plus(PPoint::new(gap, down)), self.desk.text.get("guide.none"), dim);
        }
        let mut scroll = self.scroll.get();
        scroll.fit(list.len() as f32 * row, self.frame.size.height);
        self.scroll.set(scroll);
        // How many pages each room has on its line.
        let held = lock(&self.desk.board).open();
        let open = self.desk.open.borrow();
        let origin = origin.minus(PPoint::new(0.0, scroll.at()));
        // Only the rows the part has room for are drawn.
        let shown = |index: &usize| (*index + 1) as f32 * row > scroll.at() && *index as f32 * row < scroll.at() + self.frame.size.height;
        for (index, one) in list.iter().enumerate().filter(|(index, _)| shown(index)) {
            let top = origin.y + index as f32 * row;
            match one {
                Row::Head(label) => paint.text(PPoint::new(origin.x + gap, top + down), &fit(label, width - gap * 2.0, paint), dim),
                Row::Under(label) => paint.text(PPoint::new(origin.x + gap * 1.5, top + down), &fit(label, width - gap * 2.5, paint), dim),
                Row::Room(plate) => {
                    if plate.id == *open {
                        // Against the group it stands in, as a menu's own row is.
                        light::label(paint, PRect::new(origin.x + gap / 2.0, top, width - gap, row), radius, PLight::Rest, true, "");
                    }
                    let seat = PRect::new(origin.x + gap, top + (row - icon) / 2.0, icon, icon);
                    match self.desk.site.face(&plate.with) {
                        Some(cell) => paint.picture(&cell.sheet, seat.inset(-gap / 8.0), icon, PFit::Cell { column: cell.column, index: cell.index }),
                        None => paint.icon(self.desk.icon(&plate.icon), seat, ink),
                    }
                    // How many are on its line; else how many things wait there.
                    let waits = Some(plate.count.max(self.desk.waiting(&plate.id))).filter(|count| *count > 0).map(|count| count.to_string());
                    let count = held.iter().find(|(key, ..)| *key == self.desk.key(&plate.id)).map(|(_, _, line)| line.to_string()).or(waits).unwrap_or_default();
                    let room = width - gap * 3.0 - icon - paint.measure(&count);
                    paint.text(PPoint::new(origin.x + gap * 1.5 + icon, top + down), &fit(&plate.label, room, paint), ink);
                    paint.text(PPoint::new(origin.x + width - gap - paint.measure(&count), top + down), &count, dim);
                }
            }
        }
    }
}

impl PView for Guide {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let mut scroll = self.scroll.get();
        let at = match *event {
            PEvent::PointerDown(at) => at,
            // The wheel turned away from the hand goes up the list.
            PEvent::Scroll(_, by) => {
                let any = scroll.turn(-by.y);
                self.scroll.set(scroll);
                return any;
            }
            _ => return false,
        };
        let index = ((at.y + scroll.at()) / self.desk.size(ROW)).floor().max(0.0) as usize;
        let room = match self.row().get(index) {
            Some(Row::Room(plate)) => Some(plate.id.clone()),
            _ => None,
        };
        room.is_some_and(|room| self.desk.show(&room))
    }

    fn tell(&mut self, word: &str) -> bool {
        match (word, word.strip_prefix(LOOK)) {
            (START, _) => self.desk.show(&self.desk.site.start),
            (_, Some(look)) => self.desk.dress(look),
            _ => false,
        }
    }
}
