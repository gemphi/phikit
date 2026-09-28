//! Node graph canvas — shared by Node/Shader/Compositor/GeoNode hosts.
//! Blender ref: `space_node/space_node.cc` WINDOW region draw.

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

// `BNodeItem` — the node graph canvas. Shared by Node/Shader/Compositor/GeoNode.
// Config under `ctx.<host>.zoom`, `.pan`, `.tree`, etc.
#[derive(Clone, Copy, Debug)]
pub struct BNodeItem {
    pub host: BHostType,
}

impl BNodeItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BNodeItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(self.host)
    }
}

// `BNodeList` — singleton wrapper carrying the concrete host for symmetry
// with the item/list pattern.

#[derive(Clone, Copy, Debug)]
pub struct BNodeList {
    pub host: BHostType,
}

impl BNodeList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BNodeList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BNodeItem::new(self.host).build(s)]
    }
}
