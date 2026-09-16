//! Execute bar — footer action buttons (Cancel / Open / Save …).
//! Blender ref: `RGN_TYPE_EXECUTE`.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BExecuteItem;
pub use list::BExecuteList;
pub use set::BExecuteAction;

#[derive(Clone, Copy, Debug)]
pub struct BExecute {
    pub host: BHostType,
}

impl BExecute {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BExecute {
    fn build(&self, s: &BState) -> BView {
        BView::row(BExecuteList::new(self.host).items(s))
    }
}
