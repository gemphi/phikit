use super::BPrefsItem;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BPrefsList;

impl BPrefsList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BPrefsList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BPrefsItem::new().build(s)]
    }
}
