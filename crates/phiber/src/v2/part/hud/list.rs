//! `BHudList` — iterate HUD slots for the active host. Empty by default;
//! populated when an operator finishes (Adjust Last Op etc.).

use super::{BHudItem, set::SLOTS};
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

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
