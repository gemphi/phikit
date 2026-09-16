//! `BButsItem` — property panels body. Delegates the panel layout to the
//! `Panel` part; renderer resolves via `ctx.buts.navbar.tab` + `panel::set`.

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

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
