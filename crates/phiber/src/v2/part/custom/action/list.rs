use super::BActionItem;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BActionList {
    pub host: BHostType,
}

impl BActionList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BActionList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BActionItem::new(self.host).build(s)]
    }
}
