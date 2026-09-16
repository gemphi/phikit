//! `BTextItem` — line-buffer canvas. Shared by Text/Info/Console.

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BTextItem {
    pub host: BHostType,
}

impl BTextItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BTextItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(self.host)
    }
}
