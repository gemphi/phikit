//! `BImageItem` — image/UV canvas. Shared by Image and Uv.
//! Config under `ctx.<host>.zoom`, `.pan`, `.channel`, `.overlay`.

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BImageItem {
    pub host: BHostType,
}

impl BImageItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BImageItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(self.host)
    }
}
