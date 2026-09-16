//! `BView3dItem` — the 3D canvas. Config read from `ctx.view3d.*` by the
//! renderer (shading / show_floor / show_axis_[x|y|z] / show_overlays /
//! show_gizmos / lens / clip_start / clip_end). See Blender `View3D` DNA.

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug, Default)]
pub struct BView3dItem;

impl BView3dItem {
    pub fn new() -> Self {
        Self
    }
}

impl BViewItem for BView3dItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(BHostType::View3D)
    }
}
