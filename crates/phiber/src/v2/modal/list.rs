//! `BModalList` — reads `ctx.modal`; length-0 or length-1.

use super::{BModalItem, BModalType};
use crate::v2::state::{BState, key::K_MODAL};
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BModalList;

impl BModalList {
    pub fn new() -> Self {
        Self
    }

    fn active(s: &BState) -> Option<BModalType> {
        s.get(&K_MODAL).and_then(BModalType::from_slug)
    }
}

impl BViewList for BModalList {
    fn items(&self, s: &BState) -> Vec<BView> {
        match Self::active(s) {
            Some(m) => vec![BModalItem::new(m).build(s)],
            None => vec![],
        }
    }
}
