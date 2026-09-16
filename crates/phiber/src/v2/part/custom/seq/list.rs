use super::BSeqItem;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BSeqList;

impl BSeqList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BSeqList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BSeqItem::new().build(s)]
    }
}
