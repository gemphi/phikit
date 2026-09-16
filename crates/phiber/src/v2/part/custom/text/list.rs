use super::BTextItem;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BTextList {
    pub host: BHostType,
}

impl BTextList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BTextList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BTextItem::new(self.host).build(s)]
    }
}
