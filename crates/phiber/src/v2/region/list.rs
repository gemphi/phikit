//! `BRegionList` — iterate the regions of a host.

use super::BRegionItem;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BRegionList {
    pub host: BHostType,
}

impl BRegionList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BRegionList {
    fn items(&self, state: &BState) -> Vec<BView> {
        self.host
            .spec()
            .regions
            .iter()
            .map(|r| BRegionItem::new(self.host, *r).build(state))
            .filter(|v| !v.is_empty())
            .collect()
    }
}
