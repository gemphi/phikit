//! Movie clip canvas. Blender ref: `space_clip/space_clip.cc` WINDOW region.

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

#[derive(Clone, Copy, Debug, Default)]
pub struct BClipItem;

impl BClipItem {
    pub fn new() -> Self {
        Self
    }
}

impl BViewItem for BClipItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(BHostType::Clip)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BClipList;

impl BClipList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BClipList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BClipItem::new().build(s)]
    }
}
