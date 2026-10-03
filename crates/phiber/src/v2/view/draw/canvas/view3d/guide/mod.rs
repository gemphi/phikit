//! What the viewport draws over the scene, each part as the Overlays popover
//! asks (`scene::BOverlays`, `VIEW3D_PT_overlay`): the floor and its axes
//! (`floor.rs`) under the meshes while the tracer has no image yet; then the
//! extras, bones and relationship lines (`extra.rs`), the objects' origins
//! (`origin.rs`) and the 3D cursor (`cursor.rs`) over everything, in the
//! order Blender's overlay engine draws them.
//!
//! Everything is projected here, inside the frame's camera
//! (`scene::camera::with_view`); the canvas only strokes it.

mod cursor;
mod origin;
pub mod set;
#[cfg(test)]
mod tests;

use self::cursor::{BCursorInk, BCursorMark};
use self::origin::{BOriginDot, BOriginInk};
use super::extra::{BExtraPalette, BProjectedLine};
use super::floor::BFloorPalette;
use super::mesh::BMeshPalette;
use crate::v2::scene::camera::BViewSetup;
use crate::v2::scene::pick::extra::BPickExtra;
use crate::v2::scene::select::BSelection;
use crate::v2::scene::{BObject3D, BOverlays};
use crate::v2::state::{BState, CtxKey};
use crate::v2::view::draw::color::to_hsla;
use crate::v2::view::draw::ctx::BRenderCtx;
use gpui::{Bounds, Hsla, Pixels, Window};

/// One frame's overlays, projected.
pub(super) struct BGuide {
    floor: Vec<BProjectedLine>,
    over: Vec<BProjectedLine>,
    origins: Vec<BOriginDot>,
    ring: Hsla,
    cursor: Option<BCursorMark>,
}

impl BGuide {
    /// The overlays of the view `(yaw, pitch, pan, zoom)`, `scene` the
    /// objects drawn and `extras` the cameras, lights and empties drawn.
    pub(super) fn build(
        ctx: &BRenderCtx,
        view: (f32, f32, (f32, f32), f32),
        setup: BViewSetup,
        pivot: [f32; 3],
        palette: &BMeshPalette,
        scene: &[BObject3D],
        extras: &[BPickExtra],
    ) -> Self {
        let (s, style) = (ctx.state, &ctx.style);
        let shown = BOverlays::now(s);
        let stroke = |c| to_hsla(style.axis_stroke(c).color);
        let floor_ink = BFloorPalette {
            grid: to_hsla(style.viewport_grid_stroke().color),
            axis_x: stroke(style.colors.axis_x),
            axis_y: stroke(style.colors.axis_y),
            axis_z: stroke(style.colors.axis_z),
        };
        let (yaw, pitch, pan, zoom) = view;
        let white = to_hsla(style.view_overlay());
        let origins = match shown.origins {
            true => {
                let ink = BOriginInk { active: palette.accent_active, selected: palette.accent, other: to_hsla(style.colors.text_dim) };
                let active = s.get(CtxKey::Object).unwrap_or("");
                BOriginDot::list(&BSelection::now(s), active, shown.origins_all, scene, extras, view, setup, ink)
            }
            false => Vec::new(),
        };
        let cursor = shown.cursor.then(|| {
            let ink = BCursorInk { red: to_hsla(style.colors.axis_x), white, axis: [style.colors.axis_x, style.colors.axis_y, style.colors.axis_z].map(to_hsla) };
            let matrix = crate::v2::scene::cursor::BCursor::now(s).matrix().to_cols_array_2d();
            BCursorMark::of(matrix, view, setup, ink)
        });
        Self {
            floor: super::floor::lines(view, setup, pivot, floor_ink, shown),
            over: super::extra::project(s, yaw, pitch, pan, zoom, BExtraPalette::from_style(ctx, palette)),
            origins,
            ring: white,
            cursor: cursor.flatten(),
        }
    }

    /// The selection the meshes are outlined for: none with Outline
    /// Selected off (`V3D_SELECT_OUTLINE`) or every overlay hidden.
    pub(super) fn outlined(s: &BState) -> BSelection {
        match BOverlays::now(s).outline {
            true => BSelection::now(s),
            false => BSelection::default(),
        }
    }

    /// The floor and its axes, where nothing else drew them.
    pub(super) fn paint_floor(&self, bounds: Bounds<Pixels>, window: &mut Window) {
        super::extra::paint(bounds, &self.floor, 1.0, window);
    }

    /// Everything that sits over the scene, `width` a line's.
    pub(super) fn paint_over(&self, bounds: Bounds<Pixels>, width: f32, window: &mut Window) {
        super::extra::paint(bounds, &self.over, width, window);
        BOriginDot::paint(&self.origins, self.ring, bounds, window);
        if let Some(cursor) = &self.cursor {
            cursor.paint(bounds, width, window);
        }
    }
}
