use super::BClipItem;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BClipList;

impl BClipList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BClipList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BClipItem::new().build(s)]
    }
}
