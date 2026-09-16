//! `BPaneList` — iterate the panes of a region.

use super::{BPaneItem, BPaneSpec};
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BPaneList<'a> {
    pub host: BHostType,
    pub panes: &'a [BPaneSpec],
}

impl<'a> BPaneList<'a> {
    pub fn new(host: BHostType, panes: &'a [BPaneSpec]) -> Self {
        Self { host, panes }
    }
}

impl<'a> BViewList for BPaneList<'a> {
    fn items(&self, state: &BState) -> Vec<BView> {
        self.panes
            .iter()
            .map(|p| BPaneItem::new(self.host, *p).build(state))
            .filter(|v| !v.is_empty())
            .collect()
    }
}
