//! Panel — accordion of property sections with RNA-backed props.

pub mod item;
pub mod list;
pub mod prop;
pub mod prop_view;
pub mod set;

// Per-context property tables.
pub mod collection;
pub mod constraint;
pub mod data;
pub mod material;
pub mod modifier;
pub mod object;
pub mod output;
pub mod particles;
pub mod physics;
pub mod prefs;
pub mod render;
pub mod scene;
pub mod texture;
pub mod tool;
pub mod view_layer;
pub mod world;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BPanelRole, BView, BViewItem, BViewList};

pub use item::BPanelItem;
pub use list::BPanelList;
pub use prop::{BEnumItem, BPropKind, BPropSpec, BUnit};
pub use set::{BSectionSpec, sections_for};

#[derive(Clone, Copy, Debug)]
pub struct BPanel {
    pub host: BHostType,
}

impl BPanel {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BPanel {
    fn build(&self, s: &BState) -> BView {
        BView::panel(BPanelRole::Body, BPanelList::new(self.host).items(s))
    }
}
