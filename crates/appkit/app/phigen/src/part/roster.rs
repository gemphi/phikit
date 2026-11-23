//! The roster: the models that can answer, a row each, the one that
//! answers marked. A press on another has it answer from now on, and
//! that is kept. The server says which models it has; until it has, the
//! one `data/model.tsv` names is the only row. Where no model is
//! connected it says so, and has no rows.
//!
//! It is the content of the modal the pick opens, where a choice closes
//! the modal, and a part of the settings window, where it does not.

use std::cell::Cell;

use appkit::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView, light};
use appkit::pui::layout::{PScroll, wrap};
use appkit::pui::window::PDeed;

use super::{DIM, Desk, GAP, INK, LINE, fit};

/// The theme's tokens for a row: how high it is, how round the light
/// behind the one that answers is, and how large its mark is.
pub const ROW: &str = "row.height";
pub const RADIUS: &str = "row.radius";
pub const ICON: &str = "icon.size";
/// The icon beside the model that answers.
pub const CHOSEN: &str = "check-circle";

/// The models that can answer.
pub struct Roster {
    frame: PRect,
    desk: Desk,
    /// The models, as the server calls them.
    name: Vec<String>,
    /// Whether a choice takes the modal away.
    close: bool,
    scroll: Cell<PScroll>,
    /// How far down the rows began when it was drawn last.
    under: Cell<f32>,
}

impl Roster {
    pub fn new(desk: Desk, close: bool) -> Roster {
        Roster { frame: PRect::default(), desk, name: Vec::new(), close, scroll: Cell::default(), under: Cell::default() }
    }
}

impl PDraw for Roster {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, row, line, icon) = (paint.size(GAP), paint.size(ROW), paint.size(LINE), paint.size(ICON));
        let (ink, dim, radius) = (paint.color(INK), paint.color(DIM), paint.size(RADIUS));
        let (left, wide) = (origin.x + gap, self.frame.size.width - gap * 2.0);
        // A line or two that say what the rows are, or why there are none.
        let note = match self.name.is_empty() {
            true => "roster.none",
            false => "roster.line",
        };
        let set = wrap(self.desk.text.get(note), wide, &|text| paint.measure(text));
        for (index, text) in set.iter().enumerate() {
            paint.text(PPoint::new(left, origin.y + index as f32 * line), text, dim);
        }
        // The rows scroll under it when they are more than there is room for.
        let under = set.len() as f32 * line + gap / 2.0;
        self.under.set(under);
        let room = PRect::new(origin.x, origin.y + under, self.frame.size.width, (self.frame.size.height - under).max(0.0));
        let mut scroll = self.scroll.get();
        scroll.fit(self.name.len() as f32 * row, room.size.height);
        self.scroll.set(scroll);
        let history = self.desk.history.borrow();
        paint.clip(room);
        // Only the rows there is room for are drawn.
        let shown = |index: &usize| (*index + 1) as f32 * row > scroll.at() && *index as f32 * row < scroll.at() + room.size.height;
        for (index, name) in self.name.iter().enumerate().filter(|(index, _)| shown(index)) {
            let rect = PRect::new(origin.x + gap / 2.0, room.origin.y + index as f32 * row - scroll.at(), self.frame.size.width - gap, row);
            let chosen = name == history.voice();
            if chosen {
                light::label(paint, rect, radius, PLight::Strong, false, "");
                paint.icon(CHOSEN, PRect::new(left + wide - icon, rect.center().y - icon / 2.0, icon, icon), ink);
            }
            paint.text(PPoint::new(left, rect.center().y - line / 2.0), &fit(name, wide - icon - gap, paint), ink);
        }
        paint.unclip();
    }
}

impl PView for Roster {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    /// Once a frame it asks which models there are: the server may have
    /// said since. While the server has not, another frame is drawn, so
    /// the rows are there as soon as it has.
    fn tick(&mut self) -> bool {
        let mut history = self.desk.history.borrow_mut();
        self.name = history.roster();
        history.asking()
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let mut scroll = self.scroll.get();
        match *event {
            // The wheel turned away from the hand goes up the rows.
            PEvent::Scroll(_, by) => {
                let any = scroll.turn(-by.y);
                self.scroll.set(scroll);
                any
            }
            PEvent::PointerDown(at) => {
                let row = ((at.y - self.under.get() + scroll.at()) / self.desk.size(ROW)).floor();
                let name = Some(row).filter(|row| *row >= 0.0 && at.y >= self.under.get()).and_then(|row| self.name.get(row as usize)).cloned();
                let chosen = name.is_some_and(|name| self.desk.choose(&name));
                if chosen && self.close {
                    self.desk.wish.borrow_mut().push(PDeed::Close);
                }
                chosen
            }
            _ => false,
        }
    }
}
