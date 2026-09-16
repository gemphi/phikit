use super::BButsItem;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BButsList;

impl BButsList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BButsList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BButsItem::new().build(s)]
    }
}
