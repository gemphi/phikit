//! View3D center content — the 3D canvas.
//! Blender ref: `space_view3d.cc` WINDOW region draw.

use crate::v2::area::current::{current_area, current_scope};
use crate::v2::scene::camera_view::{BCameraSlot, camera_view_key};
use crate::v2::state::BState;
use crate::v2::state::val::{V_FALSE, V_TRUE};
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

// `BView3dItem` — the 3D canvas. Config read from `ctx.view3d.*` by the
// renderer (shading / show_floor / show_axis_[x|y|z] / show_overlays /
// show_gizmos / lens / clip_start / clip_end). See Blender `View3D` DNA.
#[derive(Clone, Copy, Debug, Default)]
pub struct BView3dItem;

impl BView3dItem {
    pub fn new() -> Self {
        Self
    }
}

/// Does the viewport being built look through the scene camera? Its own
/// `ctx.view3d.<scope>.<slot>.camera` when set; otherwise what the workspace
/// opens it in (`startup.blend`: the Animation workspace's left viewport).
/// Only with a camera to look through — a scene without one shows the orbit.
pub fn looks_through_camera(state: &BState) -> bool {
    let (Some(scope), Some(area)) = (current_scope(), current_area()) else { return false };
    let saved = state.get(&camera_view_key(scope.slug(), &area));
    let camera_view = match saved {
        Some(V_TRUE) => true,
        Some(V_FALSE) => false,
        _ => match scope {
            crate::v2::area::current::BScope::Space(space) => space.opens_in_camera_view(area.kind),
            crate::v2::area::current::BScope::Modal(_) => false,
        },
    };
    camera_view && crate::v2::scene::source::with_loaded(|model| model.is_some_and(|model| !model.cameras.is_empty()))
}

impl BViewItem for BView3dItem {
    fn build(&self, state: &BState) -> BView {
        let key = current_scope().zip(current_area()).map(|(scope, area)| camera_view_key(scope.slug(), &area));
        BView::view3d_canvas(BCameraSlot { key, through: looks_through_camera(state) })
    }
}

// `BView3dList` — singleton wrapper for symmetry with the item/list pattern.

#[derive(Clone, Copy, Debug, Default)]
pub struct BView3dList;

impl BView3dList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BView3dList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BView3dItem::new().build(s)]
    }
}
