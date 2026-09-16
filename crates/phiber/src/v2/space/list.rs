//! `BSpaceList` — the app root. Emits the active `BSpaceItem` and, if
//! `ctx.modal` is set, the modal overlay as a sibling on top.

use super::{BSpaceItem, BSpaceType};
use crate::v2::modal::BModalList;
use crate::v2::state::{BState, key::K_SPACE};
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BSpaceList;

impl BSpaceList {
    pub fn new() -> Self {
        Self
    }
    pub fn active(state: &BState) -> BSpaceType {
        state
            .get(&K_SPACE)
            .and_then(BSpaceType::from_slug)
            .unwrap_or(BSpaceType::Layout)
    }
}

impl BViewList for BSpaceList {
    fn items(&self, state: &BState) -> Vec<BView> {
        let mut out = vec![BSpaceItem::new(Self::active(state)).build(state)];
        out.extend(BModalList::new().items(state));
        out
    }
}
