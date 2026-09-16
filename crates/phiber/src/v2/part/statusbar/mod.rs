//! Statusbar (part) — horizontal segments: scene • frame • stats • mem • version.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BStatusbarItem;
pub use list::BStatusbarList;
pub use set::{BStatusSegment, BStatusSegmentSpec};

#[derive(Clone, Copy, Debug)]
pub struct BStatusbar {
    pub host: BHostType,
}

impl BStatusbar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BStatusbar {
    fn build(&self, s: &BState) -> BView {
        BView::row(BStatusbarList::new().items(s))
    }
}
