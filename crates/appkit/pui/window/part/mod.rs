//! A part: the leaf, the content. `PPart`. The parts every app has live
//! here, and an app changes only their data: the caption with the app's
//! mark and the pin, the activity bar with the menus of its rows and the
//! user's button, the tabs of the spaces, and the profile.
//!
//! An app's own part is its entry here and a view the app builds
//! (`PApp::content`).

mod activity;
mod caption;
pub mod menu;
mod profile;
mod tab;

pub use activity::{PActivity, PRail};
pub use caption::{PCaption, PGrip};
pub use menu::{PGroup, PItem, PLaid, PSeat};
pub use profile::{PFact, PProfile};
pub use tab::PTab;

use super::{PDeed, PEntry, PLevel, PWindow};
use crate::pui::core::letter::letter;
use crate::pui::core::light::ink;
use crate::pui::core::{PFit, PLight, PPaint, PRect};
use crate::service::PTable;
use crate::theme::PTheme;

/// The theme's tokens every shared part is drawn with.
pub const GAP: &str = "gap";
pub const INK: &str = "text.ink";
pub const DIM: &str = "text.dim";
pub const ROW: &str = "text.line";
pub const ICON: &str = "icon.size";
/// The fill and the ink of the user's mark: a shape with their letter.
pub const FACE: &str = "user.fill";
pub const NAME: &str = "user.ink";

/// The leaf: a piece of content.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PPart {
    pub entry: PEntry,
}

impl PLevel for PPart {
    const WORD: &'static str = "part";

    fn of(entry: PEntry) -> PPart {
        PPart { entry }
    }

    fn entry(&self) -> &PEntry {
        &self.entry
    }
}

/// A bar of the window's own: the same in every app. It is given its
/// rectangle, says where a press does what, and draws itself from what
/// the window shows.
pub trait PBar {
    /// Where a press does something, and what.
    fn spot(&self, bar: PRect, window: &PWindow, theme: &dyn PTheme) -> Vec<(PRect, PDeed)>;

    fn draw(&self, bar: PRect, window: &PWindow, paint: &mut dyn PPaint);
}

/// The shape the user's mark stands on: the letter o of Phi-Soft,
/// filled, which `script/soft.py` draws from the font. Only the user's
/// mark has it; every other icon stands on a chip.
pub const SHAPE: &str = "soft-fill";

/// The row of the user's table that names their picture: the sheet it
/// is on, how many pictures the sheet has in a row, and which it is,
/// apart by spaces. A user without that row has no picture.
pub const PICTURE: &str = "face";

/// The user's picture, as their table has it.
pub fn picture(user: &PTable) -> Option<(String, u32, u32)> {
    let mut word = user.get(PICTURE).split_whitespace();
    Some((word.next()?.to_string(), word.next()?.parse().ok()?, word.next()?.parse().ok()?))
}

/// Draw the user in a square: their picture, round, where they have
/// one; their mark where they have none.
pub fn person(face: Option<&(String, u32, u32)>, name: &str, square: PRect, paint: &mut dyn PPaint) {
    match face {
        Some((sheet, column, index)) => paint.picture(sheet, square, square.size.width / 2.0, PFit::Cell { column: *column, index: *index }),
        None => mark(name, square, paint),
    }
}

/// Draw the user's mark in a square: Phi-Soft's own soft shape, in the
/// theme's light where it has one and its own fill where not, with the
/// letter their name begins with in its middle, drawn as every icon is
/// and as large as one.
pub fn mark(name: &str, square: PRect, paint: &mut dyn PPaint) {
    let (ink, side) = (ink(paint, NAME), paint.size(ICON));
    let fill = PLight::Strong.shade(paint.theme()).unwrap_or_else(|| paint.color(FACE));
    paint.icon(SHAPE, square, fill);
    let middle = square.center();
    paint.icon(&letter(name), PRect::new(middle.x - side / 2.0, middle.y - side / 2.0, side, side), ink);
}
