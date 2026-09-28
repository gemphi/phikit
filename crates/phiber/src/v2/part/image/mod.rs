//! Image/UV canvas — shared by Image and Uv hosts.
//! Blender ref: `space_image/space_image.cc` WINDOW region draw.

pub mod header;
pub mod uv;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

// `BImageItem` — image/UV canvas. Shared by Image and Uv.
// Config under `ctx.<host>.zoom`, `.pan`, `.channel`, `.overlay`.
#[derive(Clone, Copy, Debug)]
pub struct BImageItem {
    pub host: BHostType,
}

impl BImageItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BImageItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(self.host)
    }
}

// `BImageList` — singleton wrapper carrying the concrete host for symmetry
// with the item/list pattern.

#[derive(Clone, Copy, Debug)]
pub struct BImageList {
    pub host: BHostType,
}

impl BImageList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BImageList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BImageItem::new(self.host).build(s)]
    }
}
