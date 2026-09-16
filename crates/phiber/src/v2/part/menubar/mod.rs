//! Menubar — horizontal row of menu buttons.
//! Layout is driven by the host's BPaneSpec; the Switcher part separately
//! provides the editor-type icon on the left pane.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BMenubarItem;
pub use list::BMenubarList;
pub use set::BMenuSpec;

#[derive(Clone, Copy, Debug)]
pub struct BMenubar {
    pub host: BHostType,
}

impl BMenubar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BMenubar {
    fn build(&self, s: &BState) -> BView {
        BView::row(BMenubarList::new(self.host).items(s))
    }
}
