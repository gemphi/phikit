//! A shade: a theme that is one of several looks, and can be turned to
//! another while the window is open, as a person chooses light or dark.
//! What is drawn asks the theme for every colour each time it is drawn,
//! so the next frame is in the look that was turned to.

use std::cell::{Cell, RefCell};

use super::{PLook, PTheme};
use crate::pui::core::PColor;

/// A theme that is one of several looks.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PShade {
    /// The looks, each under its name.
    look: Vec<(String, PLook)>,
    /// Which one is shown.
    at: Cell<usize>,
    /// What is worn over the look shown: colours and sizes that take
    /// the place of its own, whichever look is turned to.
    over: RefCell<Option<PLook>>,
}

impl PShade {
    /// A shade of those looks, showing the first.
    pub fn new(look: Vec<(String, PLook)>) -> PShade {
        PShade { look, at: Cell::new(0), over: RefCell::default() }
    }

    /// The names of its looks, in order.
    pub fn list(&self) -> Vec<&str> {
        self.look.iter().map(|(name, _)| name.as_str()).collect()
    }

    /// The name of the look that is shown; nothing for a shade of none.
    pub fn name(&self) -> &str {
        self.look.get(self.at.get()).map_or("", |(name, _)| name.as_str())
    }

    /// Turn to the look of that name. `false`, and nothing changes, when
    /// it has none.
    pub fn turn(&self, name: &str) -> bool {
        let found = self.look.iter().position(|(known, _)| known == name);
        found.into_iter().for_each(|index| self.at.set(index));
        found.is_some()
    }

    /// Wear the rows of a table over the look shown: the colours and
    /// sizes they name take the place of the look's own, until other
    /// rows are worn. A table with no rows takes them off.
    pub fn wear(&self, text: &str) {
        *self.over.borrow_mut() = Some(text).filter(|text| !text.trim().is_empty()).map(PLook::read);
    }

    /// The look that is shown.
    fn shown(&self) -> Option<&PLook> {
        self.look.get(self.at.get()).map(|(_, look)| look)
    }
}

impl PTheme for PShade {
    fn color(&self, token: &str) -> Option<PColor> {
        let worn = self.over.borrow().as_ref().and_then(|over| over.color(token));
        worn.or_else(|| self.shown()?.color(token))
    }

    fn size(&self, token: &str) -> Option<f32> {
        let worn = self.over.borrow().as_ref().and_then(|over| over.size(token));
        worn.or_else(|| self.shown()?.size(token))
    }

    fn word(&self, token: &str) -> Option<&str> {
        self.shown()?.word(token)
    }
}
