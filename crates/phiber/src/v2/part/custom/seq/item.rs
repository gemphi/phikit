//! `BSeqItem` — sequencer timeline canvas (Blender WINDOW + PREVIEW regions).

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug, Default)]
pub struct BSeqItem;

impl BSeqItem {
    pub fn new() -> Self {
        Self
    }
}

impl BViewItem for BSeqItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(BHostType::Seq)
    }
}
