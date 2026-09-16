use super::BSpreadsheetItem;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BSpreadsheetList;

impl BSpreadsheetList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BSpreadsheetList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BSpreadsheetItem::new().build(s)]
    }
}
