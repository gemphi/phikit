//! `BFileList` — the File host's single body item.

use super::BFileItem;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BFileList {
    pub host: BHostType,
}

impl BFileList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BFileList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BFileItem::new(self.host).build(s)]
    }
}
