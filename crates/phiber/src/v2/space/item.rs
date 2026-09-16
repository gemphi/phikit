//! `BSpaceItem` — one space.

use super::BSpaceType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};
use crate::v2::zone::BZoneList;

#[derive(Clone, Copy, Debug)]
pub struct BSpaceItem {
    pub space: BSpaceType,
}

impl BSpaceItem {
    pub fn new(space: BSpaceType) -> Self {
        Self { space }
    }
}

impl BViewItem for BSpaceItem {
    fn build(&self, state: &BState) -> BView {
        // Record the active space for the whole subtree — area slots are
        // keyed per space so one workspace's editor choice cannot leak into
        // another's (see `v2::area::current`).
        crate::v2::area::with_space(self.space, || BView::Space {
            kind: self.space,
            children: BZoneList::new(self.space.spec()).items(state),
        })
    }
}
