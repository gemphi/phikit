//! The modal list — reads `ctx.modal`: the one dialog that is up, or none.
//!
//! Opening a file no longer takes the screen: its progress lives in the
//! status zone, which grows into the loading panel (`part::statusbar`) so the
//! viewport stays visible while the scene loads.

use super::{BModalItem, BModalType};
use crate::v2::state::{BState, ModalKey};
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BModalList;

impl BModalList {
    pub fn new() -> Self {
        Self
    }

    fn active(s: &BState) -> Option<BModalType> {
        s.get(ModalKey::Open).and_then(BModalType::from_slug)
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

