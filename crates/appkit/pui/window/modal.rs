//! A modal: an overlay on top of the space, a small app of its own.
//! `PModal`. The modals every app has live here: about, the command palette.
//!
//! A modal lies in the window in one of three shapes, which the app's
//! table gives it: a square in the middle, a drawer at the right edge
//! for what is done beside the page, or a sheet that comes up from the
//! foot; or it fills the window. The panel that shows it (`panel.rs`)
//! is the same in each, with a head that names it and two buttons. A
//! modal whose table says `bare` after its shape has a panel with no
//! head: its content has all of it, and its own way out.

use super::{PEntry, PLevel};

/// The word a table leaves a modal's panel without a head by.
pub const BARE: &str = "bare";

/// How a modal lies in the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PShape {
    /// A square in the middle of the window.
    #[default]
    Square,
    /// A panel at the window's right edge, as high as the window.
    Drawer,
    /// A panel at the window's foot.
    Sheet,
    /// The whole window: what is looked at alone, as a picture is.
    Whole,
}

impl PShape {
    /// The shape a table names: `drawer`, `sheet`, `whole`, and a square
    /// otherwise.
    pub fn named(word: &str) -> PShape {
        match word {
            "drawer" => PShape::Drawer,
            "sheet" => PShape::Sheet,
            "whole" => PShape::Whole,
            _ => PShape::Square,
        }
    }
}

/// An overlay on top of the space.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PModal {
    pub entry: PEntry,
    pub shape: PShape,
    /// Whether its panel has no head: no title and no buttons.
    pub bare: bool,
}

impl PLevel for PModal {
    const WORD: &'static str = "modal";

    fn of(entry: PEntry) -> PModal {
        PModal { entry, shape: PShape::default(), bare: false }
    }

    fn entry(&self) -> &PEntry {
        &self.entry
    }

    /// The cell after a modal's icon is its shape, and `bare` in the
    /// one after that leaves its panel without a head.
    fn more(&mut self, cell: &[&str]) {
        self.shape = PShape::named(cell.first().copied().unwrap_or_default());
        self.bare = cell.get(1) == Some(&BARE);
    }
}
