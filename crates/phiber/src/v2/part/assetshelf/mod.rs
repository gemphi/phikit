//! Asset Shelf — brush / preset / texture thumbnails.
//! Blender ref: `RGN_TYPE_ASSET_SHELF`.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BAssetShelfItem;
pub use list::BAssetShelfList;
pub use set::BAssetTileSpec;

#[derive(Clone, Copy, Debug)]
pub struct BAssetShelf {
    pub host: BHostType,
}

impl BAssetShelf {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BAssetShelf {
    fn build(&self, s: &BState) -> BView {
        BView::row(BAssetShelfList::new(self.host).items(s))
    }
}
