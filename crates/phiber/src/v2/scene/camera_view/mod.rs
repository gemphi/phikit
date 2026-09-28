//! Looking through the scene camera (`RegionView3D.persp == RV3D_CAMOB`).
//!
//! Blender draws the camera's frame inside the region and darkens what lies
//! outside it (`view3d_draw.cc::drawviewborder`). Where the frame lands is
//! `view3d_camera_border`: the camera's view plane (its sensor over its lens,
//! at the render resolution) placed inside the region's view plane (the same
//! lens, at the region's size, zoomed by `camzoom`). This file is that
//! geometry — structure here, the orbit-model parameters the canvas projects
//! with in `set.rs`.

pub mod set;

use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::space::BSpaceType;
use crate::v2::state::{BKey as _, BPath, View3dKey};

/// `RV3D_CAMZOOM_MIN` / `RV3D_CAMZOOM_MAX`.
pub const CAMZOOM_MIN: f32 = -30.0;
pub const CAMZOOM_MAX: f32 = 600.0;
/// `view3d_center_camera_exec`: "4px is just a little room from the edge".
const FIT_MARGIN: f32 = 4.0;
/// `Camera.passepartalpha`'s default, `DNA_camera_types.h`.
pub const PASSEPARTOUT_ALPHA: f32 = 0.5;

/// `BKE_screen_view3d_zoom_to_fac`: the view-plane factor for a `camzoom`.
pub fn zoom_to_fac(camzoom: f32) -> f32 {
    (std::f32::consts::SQRT_2 + camzoom / 50.0).powi(2) / 4.0
}

/// `BKE_screen_view3d_zoom_from_fac`.
pub fn zoom_from_fac(fac: f32) -> f32 {
    ((fac * 4.0).sqrt() - std::f32::consts::SQRT_2) * 50.0
}

/// `BKE_camera_sensor_fit` for `CAMERA_SENSOR_FIT_AUTO`: the sensor fits the
/// larger side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BSensorFit {
    Horizontal,
    Vertical,
}

impl BSensorFit {
    pub fn auto(width: f32, height: f32) -> Self {
        if width >= height { Self::Horizontal } else { Self::Vertical }
    }
}

/// The camera's frame inside a region, in region pixels with y down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BCameraFrame {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl BCameraFrame {
    /// `view3d_camera_border` for a region `region` px wide and high, a
    /// render `render` px wide and high (`Scene.r.xsch × ysch`, square
    /// pixels), and the region's `camzoom`. Both view planes are built from
    /// one lens, so only the two sensor fits and the zoom remain: the frame
    /// is `render.x × viewfac_region / (viewfac_camera × zoom)` wide, centred.
    pub fn of(region: (f32, f32), render: (f32, f32), camzoom: f32) -> Self {
        let (width, height) = Self::size(region, render, zoom_to_fac(camzoom));
        let (left, top) = ((region.0 - width) / 2.0, (region.1 - height) / 2.0);
        Self { left, top, right: left + width, bottom: top + height }
    }

    /// The frame's size at a view-plane factor (`ED_view3d_calc_camera_border_size`
    /// passes `no_zoom`, which is factor 1).
    fn size(region: (f32, f32), render: (f32, f32), fac: f32) -> (f32, f32) {
        let viewfac_region = match BSensorFit::auto(region.0, region.1) {
            BSensorFit::Horizontal => region.0,
            BSensorFit::Vertical => region.1,
        };
        let viewfac_camera = match BSensorFit::auto(render.0, render.1) {
            BSensorFit::Horizontal => render.0,
            BSensorFit::Vertical => render.1,
        };
        // `params.zoom = CAMERA_PARAM_ZOOM_INIT_CAMOB / fac` on the region side.
        let scale = viewfac_region * fac / viewfac_camera.max(1.0);
        (render.0 * scale, render.1 * scale)
    }

    /// `view3d_center_camera_exec`: the `camzoom` that fits the frame in the
    /// region with four pixels to spare, clamped to Blender's range.
    pub fn fitted_camzoom(region: (f32, f32), render: (f32, f32)) -> f32 {
        let (width, height) = Self::size(region, render, 1.0);
        let fac = (region.0 / (width + FIT_MARGIN)).min(region.1 / (height + FIT_MARGIN));
        zoom_from_fac(fac).clamp(CAMZOOM_MIN, CAMZOOM_MAX)
    }

    pub fn width(&self) -> f32 {
        self.right - self.left
    }
    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    /// `drawviewborder`'s integer offsets: the passepartout stops one pixel
    /// outside the frame so the camera's own border shows through.
    pub fn snapped(&self) -> Self {
        Self {
            left: (self.left - 1.0001).floor(),
            top: (self.top - 1.0001).floor(),
            right: (self.right + 0.9999).floor(),
            bottom: (self.bottom + 0.9999).floor(),
        }
    }
}

/// `ctx.view3d.<scope>.<slot>.camera` — whether an area's viewport looks
/// through the camera. Unset means the space's own default
/// (`BSpaceType::opens_in_camera_view`).
/// What a 3D viewport canvas knows of its camera view: the key that holds
/// whether this area looks through the camera (`View3dKey::Camera`, one per
/// space and area), and whether it does now. A canvas built outside an area
/// has no key — nothing to toggle — and never looks through the camera.
#[derive(Clone, Debug, PartialEq)]
pub struct BCameraSlot {
    pub key: Option<BPath>,
    pub through: bool,
}

impl BCameraSlot {
    pub const fn none() -> Self {
        Self { key: None, through: false }
    }

    /// The gizmo's camera button: into the camera view, or back out of it
    /// (`VIEW3D_OT_view_camera`), for this area alone.
    pub fn toggle(&self) -> Option<crate::v2::view::prim::BAction> {
        let key = self.key.clone()?;
        Some(crate::v2::view::prim::BAction::Set(key, if self.through { "false" } else { "true" }.to_string()))
    }
}

pub fn camera_view_key(scope: &'static str, area: &BAreaSpec) -> BPath {
    View3dKey::Camera(scope, area.kind.label()).path()
}

impl BSpaceType {
    /// The areas a workspace opens looking through the camera, as
    /// `startup.blend` saves them: the Animation workspace's left viewport
    /// (`RegionView3D.persp = RV3D_CAMOB`).
    pub fn opens_in_camera_view(self, area: BAreaType) -> bool {
        match self {
            Self::Animation => area == BAreaType::Left,
            Self::Layout
            | Self::Modeling
            | Self::Sculpting
            | Self::UvEditing
            | Self::TexturePaint
            | Self::Shading
            | Self::Rendering
            | Self::Compositing
            | Self::GeometryNodes
            | Self::Scripting => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_factors_invert_and_start_at_a_half() {
        assert!((zoom_to_fac(0.0) - 0.5).abs() < 1e-6, "camzoom 0 is a view plane twice the frame");
        for camzoom in [-30.0, 0.0, 37.5, 200.0, 600.0] {
            assert!((zoom_from_fac(zoom_to_fac(camzoom)) - camzoom).abs() < 1e-3);
        }
    }

    #[test]
    fn at_camzoom_zero_a_landscape_frame_is_half_the_region_wide_and_centred() {
        let frame = BCameraFrame::of((1000.0, 600.0), (1920.0, 1080.0), 0.0);
        assert!((frame.width() - 500.0).abs() < 1e-3, "{frame:?}");
        assert!((frame.height() - 500.0 * 1080.0 / 1920.0).abs() < 1e-3);
        assert!((frame.left - 250.0).abs() < 1e-3 && ((frame.top + frame.bottom) / 2.0 - 300.0).abs() < 1e-3);
    }

    #[test]
    fn a_portrait_region_fits_the_sensor_to_its_height() {
        let frame = BCameraFrame::of((600.0, 1000.0), (1920.0, 1080.0), 0.0);
        assert!((frame.width() - 500.0).abs() < 1e-3, "viewfac is the region's height: {frame:?}");
    }

    #[test]
    fn the_fitted_zoom_leaves_four_pixels_on_the_tight_side() {
        let region = (1000.0, 600.0);
        let render = (1920.0, 1080.0);
        let frame = BCameraFrame::of(region, render, BCameraFrame::fitted_camzoom(region, render));
        // At factor 1 the frame is the region's width by 562.5: the width is
        // the tight side, so it ends up four pixels short of the region.
        assert!((frame.width() - (region.0 - FIT_MARGIN)).abs() < 0.5, "{frame:?}");
        assert!(frame.top > 0.0 && frame.bottom < region.1);
        let snapped = frame.snapped();
        assert!(snapped.left <= frame.left - 1.0 && snapped.right >= frame.right);
    }

    #[test]
    fn only_the_animation_workspaces_left_viewport_opens_through_the_camera() {
        assert!(BSpaceType::Animation.opens_in_camera_view(BAreaType::Left));
        assert!(!BSpaceType::Animation.opens_in_camera_view(BAreaType::Center));
        for space in BSpaceType::ALL.iter().copied().filter(|space| *space != BSpaceType::Animation) {
            assert!(!space.opens_in_camera_view(BAreaType::Left), "{space:?}");
        }
    }
}
