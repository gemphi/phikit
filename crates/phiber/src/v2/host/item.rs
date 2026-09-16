//! `BHostItem` — one host. Emits `BView::Host` populated by `BRegionList`.

use super::BHostType;
use crate::v2::region::BRegionList;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BHostItem {
    pub host: BHostType,
}

impl BHostItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BHostItem {
    fn build(&self, state: &BState) -> BView {
        BView::Host {
            kind: self.host,
            children: BRegionList::new(self.host).items(state),
        }
    }
}
