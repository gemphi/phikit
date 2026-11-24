//! The guide: the places a person joined, each a heading with its rooms
//! under it, the open room marked. It is the part of the guide host,
//! which appkit shows in a group of the bar's menu (`data/menu.tsv`):
//! beside the bar at a desk, drawn out over the room in a hand. A room
//! with somebody on its line says how many. A press on a room opens it;
//! the word `start`, from a row of the menu, opens the room a person
//! begins in; a look chosen in the menu or the profile is kept; and the
//! word of a host's row of the bar goes to that host's first room.
//! While the person is in a voice room, that room stands at its foot
//! (`foot`), whichever page is open. Over its rows are its tabs (`tab`):
//! Recent, Favorites, Older.

mod foot;
pub mod tab;

use std::cell::{Cell, RefCell};

pub use foot::Foot;

use appkit::pui::core::{PDraw, PEvent, PFit, PLight, PPaint, PPoint, PRect, PView, figure, light};
use appkit::pui::layout::PScroll;

use super::{DIM, Desk, GAP, ICON, INK, LINE, RADIUS, ROW, Rung, fit};
use crate::board::lock;

/// The word a row of the menu tells to open the room a person begins
/// in; and the word before a look's name when one was chosen.
pub const START: &str = "start";
pub const LOOK: &str = "look ";
/// What the profile tells, before all that is preferred.
pub const PREFER: &str = "prefer ";

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
    /// The voice room the person is in, under the rows; and where the
    /// guide was drawn last, in the window.
    foot: Foot,
    origin: Cell<PPoint>,
    /// The row the pointer is over.
    over: Cell<Option<usize>>,
    /// Where the pointer is, and where each tab was drawn last, in the
    /// window.
    point: Cell<Option<PPoint>>,
    tab: RefCell<Vec<(PRect, String)>>,
}

impl Guide {
    pub fn new(desk: Desk) -> Guide {
        Guide { frame: PRect::default(), foot: Foot::new(desk.clone()), desk, scroll: Cell::default(), origin: Cell::default(), over: Cell::default(), point: Cell::default(), tab: RefCell::default() }
    }

    /// Every row, in order.
    fn row(&self) -> Vec<Row> {
        let mut list = Vec::new();
        for (label, rung) in self.desk.sifted() {
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
        // The foot has the lowest part, the tabs the highest; the rows
        // have what is between them.
        self.origin.set(origin);
        self.foot.draw(paint, PRect { origin, size: self.frame.size });
        let chosen = self.desk.tabs().1;
        *self.tab.borrow_mut() = self.tabs(paint, origin);
        let head = self.head();
        let origin = origin.plus(PPoint::new(0.0, head));
        let high = self.frame.size.height - self.foot.high() - head;
        let list = self.row();
        if list.is_empty() {
            let none = match chosen.is_empty() {
                true => "guide.none".to_string(),
                false => format!("guide.none.{chosen}"),
            };
            return paint.text(origin.plus(PPoint::new(gap, down)), &fit(self.desk.text.get(&none), width - gap * 2.0, paint), dim);
        }
        let mut scroll = self.scroll.get();
        scroll.fit(list.len() as f32 * row, high);
        self.scroll.set(scroll);
        paint.clip(PRect::new(origin.x, origin.y, width, high));
        // How many pages each room has on its line.
        let held = lock(&self.desk.board).open();
        let open = self.desk.open.borrow();
        let origin = origin.minus(PPoint::new(0.0, scroll.at()));
        // Only the rows the part has room for are drawn.
        let shown = |index: &usize| (*index + 1) as f32 * row > scroll.at() && *index as f32 * row < scroll.at() + high;
        // Each run of rooms under a heading is a group, on a ground of
        // its own lit from the right, against the column's (lit from the
        // left); the open room is lit against that, from the left.
        let mut from = None;
        for index in 0..=list.len() {
            match (matches!(list.get(index), Some(Row::Room(_))), from) {
                (true, None) => from = Some(index),
                (false, Some(first)) => {
                    light::label(paint, PRect::new(origin.x + gap / 4.0, origin.y + first as f32 * row, width - gap / 2.0, (index - first) as f32 * row), radius, PLight::Soft, true, "");
                    from = None;
                }
                _ => {}
            }
        }
        for (index, one) in list.iter().enumerate().filter(|(index, _)| shown(index)) {
            let top = origin.y + index as f32 * row;
            match one {
                Row::Head(label) => paint.text(PPoint::new(origin.x + gap, top + down), &fit(label, width - gap * 2.0, paint), dim),
                Row::Under(label) => paint.text(PPoint::new(origin.x + gap * 1.5, top + down), &fit(label, width - gap * 2.5, paint), dim),
                Row::Room(plate) => {
                    // Against the group it stands in, as a menu's own
                    // row is; more softly under the pointer.
                    let place = PRect::new(origin.x + gap / 2.0, top, width - gap, row);
                    match (plate.id == *open, self.over.get() == Some(index)) {
                        (true, _) => light::label(paint, place, radius, PLight::Rest, false, ""),
                        (false, true) => light::label(paint, place, radius, PLight::Soft, false, ""),
                        (false, false) => {}
                    }
                    let seat = PRect::new(origin.x + gap, top + (row - icon) / 2.0, icon, icon);
                    match self.desk.site.face(&plate.with) {
                        Some(cell) => paint.picture(&cell.sheet, seat.inset(-gap / 8.0), icon, PFit::Cell { column: cell.column, index: cell.index }),
                        None => paint.icon(self.desk.icon(&plate.icon), seat, ink),
                    }
                    // How many are on its line; else how many things wait there.
                    let waits = Some(plate.count.max(self.desk.waiting(&plate.id))).filter(|count| *count > 0).map(|count| count.to_string());
                    let count = held.iter().find(|(key, ..)| *key == self.desk.key(&plate.id)).map(|(_, _, line)| line.to_string()).or(waits).unwrap_or_default();
                    // The count stands on a ground of its own, which runs
                    // against the row's where the row is the open one.
                    let room = width - gap * 3.0 - icon - figure::size(paint, &count).width;
                    paint.text(PPoint::new(origin.x + gap * 1.5 + icon, top + down), &fit(&plate.label, room, paint), ink);
                    figure::end(paint, PPoint::new(origin.x + width - gap, top + row / 2.0), &count, plate.id != *open);
                }
            }
        }
        paint.unclip();
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
            // The foot keeps where it was drawn in the window; a press
            // comes in this part's own measure.
            PEvent::PointerDown(at) if self.foot.press(at.plus(self.origin.get())) => return true,
            // A press on a tab chooses it, and the rows begin again.
            PEvent::PointerDown(at) if at.y < self.head() => {
                let tab = self.tab.borrow().iter().find(|(rect, _)| rect.contains(at.plus(self.origin.get()))).cloned();
                tab.iter().for_each(|(_, name)| self.desk.sift(name));
                self.scroll.set(PScroll::default());
                return tab.is_some();
            }
            PEvent::PointerDown(at) if at.y < self.frame.size.height - self.foot.high() => PPoint::new(at.x, at.y - self.head()),
            PEvent::PointerMove(at) => {
                let index = ((at.y - self.head() + scroll.at()) / self.desk.size(ROW)).floor().max(0.0) as usize;
                let inside = at.y >= self.head() && at.y < self.frame.size.height - self.foot.high();
                let row = Some(index).filter(|index| inside && matches!(self.row().get(*index), Some(Row::Room(_))));
                let place = at.plus(self.origin.get());
                let tab = |point: Option<PPoint>| point.and_then(|point| self.tab.borrow().iter().position(|(rect, _)| rect.contains(point)));
                let turned = self.over.replace(row) != row || tab(self.point.get()) != tab(Some(place));
                self.point.set(Some(place));
                return self.foot.hover(place) || turned;
            }
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

    /// Its rooms' rows, where they are shown.
    fn hand(&self) -> Vec<PRect> {
        let (row, head, origin) = (self.desk.size(ROW), self.head(), self.origin.get());
        let high = self.frame.size.height - self.foot.high() - head;
        let top = |index: usize| index as f32 * row - self.scroll.get().at();
        let room = self.row().into_iter().enumerate().filter(|(index, one)| matches!(one, Row::Room(_)) && top(*index) >= 0.0 && top(*index) + row <= high);
        let room = room.map(|(index, _)| PRect::new(origin.x, origin.y + head + top(index), self.frame.size.width, row));
        room.chain(self.tab.borrow().iter().map(|(rect, _)| *rect)).chain(self.foot.hands()).collect()
    }

    fn tell(&mut self, word: &str) -> bool {
        match (word, word.strip_prefix(LOOK), word.strip_prefix(super::host::GO)) {
            (START, ..) => self.desk.show(&self.desk.site.start),
            (_, Some(look), _) => self.desk.dress(look),
            (_, _, _) if word.starts_with(PREFER) => self.desk.prefer(&word[PREFER.len()..]),
            (_, _, Some(host)) => self.desk.visit(host),
            _ => false,
        }
    }
}
