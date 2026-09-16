//! `BView3dList` — singleton wrapper for symmetry with the item/list pattern.

use super::BView3dItem;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BView3dList;

impl BView3dList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BView3dList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BView3dItem::new().build(s)]
    }
}
