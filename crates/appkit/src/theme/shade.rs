//! A shade: a theme that is one of several looks, and can be turned to
//! another while the window is open, as a person chooses light or dark.
//! What is drawn asks the theme for every colour each time it is drawn,
//! so the next frame is in the look that was turned to.

use std::cell::{Cell, RefCell};

use super::{PLook, PTheme};
use crate::pui::core::PColor;

/// A preference: its kind (`size`, `color`), its token, its value.
type Liked = (String, String, String);

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
    /// What the person prefers, under what is worn and over the look:
    /// each row its kind, its token and its value; and as a look.
    prefer: RefCell<(Vec<Liked>, Option<PLook>)>,
}

impl PShade {
    /// A shade of those looks, showing the first.
    pub fn new(look: Vec<(String, PLook)>) -> PShade {
        PShade { look, at: Cell::new(0), over: RefCell::default(), prefer: RefCell::default() }
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

    /// What the person prefers, as it is kept: `kind token value` rows,
    /// apart by `;`. Nothing takes every preference off.
    pub fn prefer(&self, kept: &str) {
        let row = kept.split(';').filter_map(|one| {
            let mut part = one.split_whitespace();
            Some((part.next()?.to_string(), part.next()?.to_string(), part.next()?.to_string()))
        });
        self.preferred(row.collect());
    }

    /// Prefer a value for a token of a kind (`size`, `color`) from now on.
    pub fn put(&self, kind: &str, token: &str, value: &str) {
        let mut row: Vec<Liked> = self.prefer.borrow().0.iter().filter(|(_, known, _)| known != token).cloned().collect();
        row.push((kind.to_string(), token.to_string(), value.to_string()));
        self.preferred(row);
    }

    /// What is preferred, as it is kept.
    pub fn kept(&self) -> String {
        self.prefer.borrow().0.iter().map(|(kind, token, value)| format!("{kind} {token} {value}")).collect::<Vec<_>>().join(";")
    }

    fn preferred(&self, row: Vec<Liked>) {
        let table: String = row.iter().map(|(kind, token, value)| format!("{kind}\t{token}\t{value}\n")).collect();
        let look = Some(table).filter(|table| !table.is_empty()).map(|table| PLook::read(&table));
        *self.prefer.borrow_mut() = (row, look);
    }

    /// The look that is shown.
    fn shown(&self) -> Option<&PLook> {
        self.look.get(self.at.get()).map(|(_, look)| look)
    }

    /// A colour as it is worn, preferred or the look's own, untinted.
    fn plain(&self, token: &str) -> Option<PColor> {
        let worn = self.over.borrow().as_ref().and_then(|over| over.color(token));
        let liked = || self.prefer.borrow().1.as_ref().and_then(|prefer| prefer.color(token));
        worn.or_else(liked).or_else(|| self.shown()?.color(token))
    }
}

/// The tokens of a look that say which colours are mixed with the
/// light's own colour, and how much of it; and the light's colour.
pub const TINT_TOKEN: &str = "tint.token";
pub const TINT_SHARE: &str = "tint.share";
const LIGHT: &str = "shade.light";

impl PTheme for PShade {
    /// A colour, as worn, preferred or the look's own; and where the
    /// look names it among its tinted ones (`tint.token`), mixed with
    /// the light's own colour by the look's share (`tint.share`): so a
    /// dark look is a dark of the theme's colour, never a bare black,
    /// whatever colour the person gave the light.
    fn color(&self, token: &str) -> Option<PColor> {
        let own = self.plain(token)?;
        let share = self.size(TINT_SHARE).unwrap_or_default().clamp(0.0, 1.0);
        let tinted = share > 0.0 && self.word(TINT_TOKEN).is_some_and(|word| word.split_whitespace().any(|one| one == token));
        match (tinted, self.plain(LIGHT)) {
            (true, Some(light)) => Some(PColor { r: own.r + (light.r - own.r) * share, g: own.g + (light.g - own.g) * share, b: own.b + (light.b - own.b) * share, a: own.a }),
            _ => Some(own),
        }
    }

    fn size(&self, token: &str) -> Option<f32> {
        let worn = self.over.borrow().as_ref().and_then(|over| over.size(token));
        let liked = || self.prefer.borrow().1.as_ref().and_then(|prefer| prefer.size(token));
        worn.or_else(liked).or_else(|| self.shown()?.size(token))
    }

    fn word(&self, token: &str) -> Option<&str> {
        self.shown()?.word(token)
    }
}
