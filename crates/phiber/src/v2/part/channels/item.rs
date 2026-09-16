//! `BChannelsItem` — one channel row; carries a `BChannelSpec`.

use super::set::BChannelSpec;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BChannelsItem {
    pub spec: BChannelSpec,
}

impl BChannelsItem {
    pub fn new(spec: BChannelSpec) -> Self {
        Self { spec }
    }
}

impl BViewItem for BChannelsItem {
    fn build(&self, _s: &BState) -> BView {
        BView::label(self.spec.label)
    }
}
