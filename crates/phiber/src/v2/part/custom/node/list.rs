//! `BNodeList` — singleton wrapper carrying the concrete host for symmetry
//! with the item/list pattern.

use super::BNodeItem;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BNodeList {
    pub host: BHostType,
}

impl BNodeList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BNodeList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BNodeItem::new(self.host).build(s)]
    }
}
