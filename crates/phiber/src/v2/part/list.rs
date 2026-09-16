//! `BPartList` — iterate the parts of a pane.

use super::{BPartItem, BPartType};
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BPartList<'a> {
    pub host: BHostType,
    pub parts: &'a [BPartType],
}

impl<'a> BPartList<'a> {
    pub fn new(host: BHostType, parts: &'a [BPartType]) -> Self {
        Self { host, parts }
    }
}

impl<'a> BViewList for BPartList<'a> {
    fn items(&self, state: &BState) -> Vec<BView> {
        self.parts
            .iter()
            .map(|&p| BPartItem::new(self.host, p).build(state))
            .collect()
    }
}
