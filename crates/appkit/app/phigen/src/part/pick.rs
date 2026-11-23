//! The pick: a pill over the chat that names the model that answers. A
//! press on it opens the modal where another is chosen. Where no model
//! is connected it says so.
//!
//! It is drawn by the part that holds it, which says where; it keeps
//! where it was drawn, so a press can be told to be on it.

use std::cell::Cell;

use appkit::pui::core::{PLight, PPaint, PPoint, PRect, light};
use appkit::pui::window::PDeed;

use super::{Desk, GAP, INK, LINE};

/// The theme's tokens for the pill: how round it is, and how large its
/// icon is. It wears the label's light.
pub const ROUND: &str = "tab.radius";
pub const ICON: &str = "icon.size";
/// The icon that says a press opens something.
pub const MORE: &str = "chevron-down";

/// A pill that names the model that answers.
pub struct Pick {
    desk: Desk,
    /// The modal a press opens.
    modal: String,
    /// Where it was drawn last, in the window.
    seen: Cell<PRect>,
}

impl Pick {
    pub fn new(desk: Desk, modal: &str) -> Pick {
        Pick { desk, modal: modal.to_string(), seen: Cell::default() }
    }

    /// What it reads: the model's name, or that there is none.
    pub fn label(&self) -> String {
        let history = self.desk.history.borrow();
        match history.voice() {
            "" => self.desk.text.get("model.none").to_string(),
            name => name.to_string(),
        }
    }

    /// Draw it with its left edge at `at.x` and its middle at `at.y`.
    /// How wide it was drawn.
    pub fn draw(&self, paint: &mut dyn PPaint, at: PPoint) -> f32 {
        let (gap, line, icon, round) = (paint.size(GAP), paint.size(LINE), paint.size(ICON), paint.size(ROUND));
        let (ink, label) = (paint.color(INK), self.label());
        let rect = PRect::new(at.x, at.y - (line + gap) / 2.0, paint.measure(&label) + icon + gap * 2.5, line + gap);
        light::label(paint, rect, round, PLight::Rest, false, "");
        paint.text(PPoint::new(rect.origin.x + gap, at.y - line / 2.0), &label, ink);
        paint.icon(MORE, PRect::new(rect.origin.x + rect.size.width - gap - icon, at.y - icon / 2.0, icon, icon), ink);
        self.seen.set(rect);
        rect.size.width
    }

    /// A press at a place in the window: on the pill, it asks for the
    /// modal. `false` for a press anywhere else.
    pub fn press(&self, at: PPoint) -> bool {
        let on = self.seen.get().contains(at);
        if on {
            self.desk.wish.borrow_mut().push(PDeed::Open(self.modal.clone()));
        }
        on
    }
}
