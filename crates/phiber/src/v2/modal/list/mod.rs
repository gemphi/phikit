//! The modal list — reads `ctx.modal`: the one dialog that is up, or none.
//! Reading a file takes the screen whatever else was open.

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
        // Reading a file takes the screen: whatever else was open, the user
        // wants to see what is happening (and be able to cancel).
        if crate::v2::scene::view_load_progress(s).is_some() {
            return Some(BModalType::Progress);
        }
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

