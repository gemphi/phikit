use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug, Default)]
pub struct BClipItem;

impl BClipItem {
    pub fn new() -> Self {
        Self
    }
}

impl BViewItem for BClipItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(BHostType::Clip)
    }
}
