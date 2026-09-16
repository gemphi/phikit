use super::BTimeItem;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BTimeList;

impl BTimeList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BTimeList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BTimeItem::new().build(s)]
    }
}
