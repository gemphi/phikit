//! HUD — floating overlay on top of a viewport canvas.
//! Blender ref: `RGN_TYPE_HUD`.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BHudItem;
pub use list::BHudList;
pub use set::BHudSlot;

#[derive(Clone, Copy, Debug)]
pub struct BHud {
    pub host: BHostType,
}

impl BHud {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BHud {
    fn build(&self, s: &BState) -> BView {
        BView::column(BHudList::new().items(s))
    }
}
