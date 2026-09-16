//! Sidebar — vertical tab strip.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BSidebarItem;
pub use list::BSidebarList;
pub use set::BSidebarTabSpec;

#[derive(Clone, Copy, Debug)]
pub struct BSidebar {
    pub host: BHostType,
}

impl BSidebar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BSidebar {
    fn build(&self, s: &BState) -> BView {
        BSidebarList::new(self.host).rail(s)
    }
}
