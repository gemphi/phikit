//! Tool Properties — settings shown alongside the mode bar for the active tool.
//! Blender ref: `RGN_TYPE_TOOL_HEADER` content.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BToolPropsItem;
pub use list::BToolPropsList;
pub use set::BToolPropSpec;

#[derive(Clone, Copy, Debug)]
pub struct BToolProps {
    pub host: BHostType,
}

impl BToolProps {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BToolProps {
    fn build(&self, s: &BState) -> BView {
        BView::row(BToolPropsList::new(self.host).items(s))
    }
}
