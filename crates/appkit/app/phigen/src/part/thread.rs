//! The thread: every chat by its title, the open one marked, under a
//! heading. It is the part of the history host, which appkit shows in a
//! group of the bar's menu (`data/menu.tsv`): beside the bar at a desk,
//! drawn out over the chat in a hand. A press on a title opens that
//! chat; the word `begin`, from a row of the menu, begins one. A list
//! longer than its room is scrolled with the wheel.

use std::cell::Cell;

use appkit::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView, light};
use appkit::pui::layout::PScroll;

use super::{DIM, Desk, GAP, INK, LINE, fit};

/// The theme's tokens for a row: its height, and how round the light
/// behind the open one is.
pub const ROW: &str = "row.height";
pub const RADIUS: &str = "row.radius";

/// The word the bar's row tells to begin a chat.
pub const BEGIN: &str = "begin";

/// Every chat by its title.
pub struct Thread {
    frame: PRect,
    desk: Desk,
    /// How far down the list it is scrolled.
    scroll: Cell<PScroll>,
}

impl Thread {
    pub fn new(desk: Desk) -> Thread {
        Thread { frame: PRect::default(), desk, scroll: Cell::default() }
    }
}

impl PDraw for Thread {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, row, line, width) = (paint.size(GAP), paint.size(ROW), paint.size(LINE), self.frame.size.width);
        let (ink, dim, radius) = (paint.color(INK), paint.color(DIM), paint.size(RADIUS));
        let down = (row - line) / 2.0;
        let history = self.desk.history.borrow();
        // The heading and the titles scroll together when they are more
        // than the part has room for.
        let mut scroll = self.scroll.get();
        scroll.fit((history.list().len() + 1) as f32 * row, self.frame.size.height);
        self.scroll.set(scroll);
        let origin = origin.minus(PPoint::new(0.0, scroll.at()));
        paint.text(origin.plus(PPoint::new(gap, down)), self.desk.text.get("thread.head"), dim);
        // Only the titles the part has room for are drawn.
        let shown = |index: &usize| (*index + 2) as f32 * row > scroll.at() && (*index + 1) as f32 * row < scroll.at() + self.frame.size.height;
        for (index, chat) in history.list().iter().enumerate().filter(|(index, _)| shown(index)) {
            let top = origin.y + (index + 1) as f32 * row;
            if index == history.index() {
                // Against the group it stands in, as a menu's own row is.
                light::label(paint, PRect::new(origin.x + gap / 2.0, top, width - gap, row), radius, PLight::Rest, true, "");
            }
            let (title, color) = match chat.title() {
                Some(title) => (title, ink),
                None => (self.desk.text.get("thread.empty"), dim),
            };
            let shown = fit(title, width - gap * 2.0, paint);
            paint.text(PPoint::new(origin.x + gap, top + down), &shown, color);
        }
    }
}

impl PView for Thread {
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
        // The heading is the first row; the chats are under it.
        let row = ((at.y + scroll.at()) / self.desk.size(ROW)).floor() - 1.0;
        row >= 0.0 && self.desk.open(row as usize)
    }

    fn tell(&mut self, word: &str) -> bool {
        let begun = word == BEGIN;
        if begun {
            self.desk.begin();
        }
        begun
    }
}
