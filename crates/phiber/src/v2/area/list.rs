//! `BAreaList` — iterate the areas of a zone.

use super::{BAreaItem, BAreaSpec};
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BAreaList<'a> {
    pub areas: &'a [BAreaSpec],
}

impl<'a> BAreaList<'a> {
    pub fn new(areas: &'a [BAreaSpec]) -> Self {
        Self { areas }
    }
}

impl<'a> BViewList for BAreaList<'a> {
    fn items(&self, state: &BState) -> Vec<BView> {
        self.areas
            .iter()
            .map(|a| BAreaItem::new(*a).build(state))
            .collect()
    }
}
