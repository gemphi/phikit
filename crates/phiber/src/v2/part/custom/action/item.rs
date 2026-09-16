//! `BActionItem` — animation editor canvas. Shared by Action/Nla/Ipo/Drivers.

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BActionItem {
    pub host: BHostType,
}

impl BActionItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BActionItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(self.host)
    }
}
