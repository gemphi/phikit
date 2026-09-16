//! `BPaneItem` — one pane.

use super::BPaneSpec;
use crate::v2::host::BHostType;
use crate::v2::part::BPartList;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BPaneItem {
    pub host: BHostType,
    pub pane: BPaneSpec,
}

impl BPaneItem {
    pub fn new(host: BHostType, pane: BPaneSpec) -> Self {
        Self { host, pane }
    }
}

impl BViewItem for BPaneItem {
    fn build(&self, state: &BState) -> BView {
        if self.pane.parts.is_empty() {
            return BView::empty();
        }
        BView::Pane {
            kind: self.pane.kind,
            children: BPartList::new(self.host, self.pane.parts).items(state),
        }
    }
}
