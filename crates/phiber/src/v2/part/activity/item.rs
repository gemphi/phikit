//! `BActivityItem` — one activity entry (icon button).

use super::set::BActivityEntry;
use crate::v2::state::{
    BState,
    key::{K_ACTIVITY_TAB, K_MODAL},
};
use crate::v2::view::prim::BAction;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BActivityItem {
    pub entry: BActivityEntry,
    pub active: bool,
}

impl BActivityItem {
    pub fn new(entry: BActivityEntry, active: bool) -> Self {
        Self { entry, active }
    }
}

impl BViewItem for BActivityItem {
    fn build(&self, _s: &BState) -> BView {
        // Entries with a modal open it directly; entries without a modal fall
        // back to tracking `K_ACTIVITY_TAB` so the visual active state still
        // updates. Only one BAction fires per click, so we pick the modal
        // when both are present — modal open is the observable behavior.
        let action = if let Some(modal) = self.entry.modal {
            BAction::Set(K_MODAL, modal.to_string())
        } else {
            BAction::Set(K_ACTIVITY_TAB, self.entry.label.to_string())
        };
        BView::tab_with_tooltip(self.entry.icon, self.entry.label, self.active, action)
    }
}
