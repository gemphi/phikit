//! `BAssetShelfItem` — one brush tile.

use super::set::BAssetTileSpec;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BAssetShelfItem {
    pub spec: BAssetTileSpec,
    pub active: bool,
}

impl BAssetShelfItem {
    pub fn new(spec: BAssetTileSpec, active: bool) -> Self {
        Self { spec, active }
    }
}

impl BViewItem for BAssetShelfItem {
    fn build(&self, _s: &BState) -> BView {
        BView::button(self.spec.thumb)
    }
}
