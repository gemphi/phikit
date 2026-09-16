//! `BSpacebarItem` — one workspace tab. Click sets `ctx.space` to this
//! space's slug; the app rebuilds the whole tree from state.

use crate::v2::space::BSpaceType;
use crate::v2::state::{BState, key::K_SPACE};
use crate::v2::view::prim::BAction;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BSpacebarItem {
    pub space: BSpaceType,
    pub active: bool,
}

impl BSpacebarItem {
    pub fn new(space: BSpaceType, active: bool) -> Self {
        Self { space, active }
    }
}

impl BViewItem for BSpacebarItem {
    fn build(&self, _s: &BState) -> BView {
        let action = BAction::Set(K_SPACE, self.space.slug().to_string());
        // A labeled tab, not a flat button: the active workspace gets the
        // top-rounded gradient cap and the accent underline, and hovering an
        // inactive one previews the same cap.
        BView::tab_labeled(self.space.icon(), self.space.label(), self.active, action)
    }
}
