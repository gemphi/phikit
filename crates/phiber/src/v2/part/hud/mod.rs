//! HUD — floating overlay on top of a viewport canvas.
//! Blender ref: `RGN_TYPE_HUD`.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use self::set::SLOTS;
pub use set::BHudSlot;

// `BHudItem` — one HUD slot.
#[derive(Clone, Copy, Debug)]
pub struct BHudItem {
    pub slot: BHudSlot,
}

impl BHudItem {
    pub fn new(slot: BHudSlot) -> Self {
        Self { slot }
    }
}

impl BViewItem for BHudItem {
    fn build(&self, _s: &BState) -> BView {
        BView::label(self.slot.label)
    }
}

// `BHudList` — iterate HUD slots for the active host. Empty by default;
// populated when an operator finishes (Adjust Last Op etc.).

#[derive(Clone, Copy, Debug, Default)]
pub struct BHudList;

impl BHudList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BHudList {
    fn items(&self, s: &BState) -> Vec<BView> {
        SLOTS
            .iter()
            .map(|slot| BHudItem::new(*slot).build(s))
            .collect()
    }
}

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
