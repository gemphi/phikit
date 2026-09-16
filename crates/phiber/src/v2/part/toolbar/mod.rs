//! Toolbar — vertical column of tool icons.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BToolbarItem;
pub use list::BToolbarList;
pub use set::BToolSpec;

#[derive(Clone, Copy, Debug)]
pub struct BToolbar {
    pub host: BHostType,
}

impl BToolbar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BToolbar {
    fn build(&self, s: &BState) -> BView {
        BToolbarList::new(self.host).rail(s)
    }
}
