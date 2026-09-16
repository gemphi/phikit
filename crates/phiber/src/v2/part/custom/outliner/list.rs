use super::BOutlinerItem;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BOutlinerList;

impl BOutlinerList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BOutlinerList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BOutlinerItem::new().build(s)]
    }
}
