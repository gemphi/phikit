//! `BHudItem` — one HUD slot.

use super::set::BHudSlot;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

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
