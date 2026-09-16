//! Activity Bar — Phiber-native leftmost vertical navigation.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BActivityItem;
pub use list::BActivityList;
pub use set::{BActivityEntry, BActivityGroup};

#[derive(Clone, Copy, Debug)]
pub struct BActivity {
    pub host: BHostType,
}

impl BActivity {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BActivity {
    fn build(&self, s: &BState) -> BView {
        BActivityList::new().rail(s)
    }
}
