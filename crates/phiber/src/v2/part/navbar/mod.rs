//! NavBar — vertical column of context tabs (Properties).

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BNavBarItem;
pub use list::BNavBarList;
pub use set::BNavTabSpec;

#[derive(Clone, Copy, Debug)]
pub struct BNavBar {
    pub host: BHostType,
}

impl BNavBar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BNavBar {
    fn build(&self, s: &BState) -> BView {
        BNavBarList::new(self.host).rail(s)
    }
}
