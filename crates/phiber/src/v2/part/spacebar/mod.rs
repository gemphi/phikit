//! Spacebar — workspace tabs at the top center.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BSpacebarItem;
pub use list::BSpacebarList;

#[derive(Clone, Copy, Debug)]
pub struct BSpacebar {
    pub host: BHostType,
}

impl BSpacebar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BSpacebar {
    fn build(&self, s: &BState) -> BView {
        BView::row(BSpacebarList::new().items(s))
    }
}
