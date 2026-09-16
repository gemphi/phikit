//! `BImageList` — singleton wrapper carrying the concrete host for symmetry
//! with the item/list pattern.

use super::BImageItem;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BImageList {
    pub host: BHostType,
}

impl BImageList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BImageList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BImageItem::new(self.host).build(s)]
    }
}
