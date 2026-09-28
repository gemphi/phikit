//! Properties (Buts) center content — the panel body.
//! Blender ref: `space_buttons.cc` WINDOW region draw.

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

// `BButsItem` — property panels body. Delegates the panel layout to the
// `Panel` part; renderer resolves via `ctx.buts.navbar.tab` + `props::set`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BButsItem;

impl BButsItem {
    pub fn new() -> Self {
        Self
    }
}

impl BViewItem for BButsItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(BHostType::Buts)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BButsList;

impl BButsList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BButsList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BButsItem::new().build(s)]
    }
}
