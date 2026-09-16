//! `BNodeItem` — the node graph canvas. Shared by Node/Shader/Compositor/GeoNode.
//! Config under `ctx.<host>.zoom`, `.pan`, `.tree`, etc.

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BNodeItem {
    pub host: BHostType,
}

impl BNodeItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BNodeItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(self.host)
    }
}
