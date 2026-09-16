//! `BExecuteItem` — one Execute button.

use super::set::BExecuteAction;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BExecuteItem {
    pub action: BExecuteAction,
}

impl BExecuteItem {
    pub fn new(action: BExecuteAction) -> Self {
        Self { action }
    }
}

impl BViewItem for BExecuteItem {
    fn build(&self, _s: &BState) -> BView {
        BView::button(self.action.label)
    }
}
