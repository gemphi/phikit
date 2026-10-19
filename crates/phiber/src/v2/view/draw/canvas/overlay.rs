//! View3D top-left perspective and active-object overlay.
//!
//! Lines rendered here are fed entirely from state + Scene_Source, matching
//! Blender's viewport info line ("User Perspective" / "(Scene) Collection |
//! <object>" / "Frame N" / "Verts: V | Faces: F"). No values are hard-coded.

use crate::v2::state::{CtxKey};
use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use crate::v2::scene::stats_pairs;
use super::super::pair;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

pub struct BOverlay {
    /// `view3d_get_name`: "User Perspective", "Camera Orthographic", …
    pub view_name: &'static str,
}

/// `view3d_draw.cc::view3d_get_name` for a user view or a camera view:
/// `camera` is `Some(cam.type == CAM_PERSP)` when looking through one.
pub fn view_name(camera: Option<bool>, perspective: bool) -> &'static str {
    match (camera, perspective) {
        (Some(true), _) => "Camera Perspective",
        (Some(false), _) => "Camera Orthographic",
        (None, true) => "User Perspective",
        (None, false) => "User Orthographic",
    }
}

thread_local! {
    /// Why the viewport is not drawing the scene, set by the canvas each
    /// frame: the GPU declined and the CPU painter is past its ceiling.
    static NOT_DRAWN: std::cell::Cell<Option<(super::view3d::rendered::BRayDeclined, usize)>> =
        const { std::cell::Cell::new(None) };
}

/// Record whether anything is drawing the scene this frame.
pub(super) fn note_gpu(declined: Option<super::view3d::rendered::BRayDeclined>, tris: usize) {
    NOT_DRAWN.with(|n| n.set(declined.map(|d| (d, tris))));
}

impl BOverlay {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let object = ctx.state.get(CtxKey::Object).unwrap_or("Cube");
        let scene = ctx.state.get(CtxKey::Scene).unwrap_or("Scene");
        let frame = ctx.state.get(CtxKey::Frame).unwrap_or("1");
        // The status bar shows the same counts (`scene::stats_line`), so
        // the two read-outs cannot disagree about how big the scene is.
        let stats = stats_pairs(ctx.state);
        // Text Info and Statistics, as the Overlays popover has them.
        let shown = crate::v2::scene::BOverlays::now(ctx.state);
        let not_drawn = NOT_DRAWN.with(|n| n.get()).map(|(why, tris)| match why {
            super::view3d::rendered::BRayDeclined::NoGpu => format!(
                "No GPU adapter — {:.1}M triangles is more than the CPU painter will draw",
                tris as f32 / 1e6
            ),
            super::view3d::rendered::BRayDeclined::TooLarge => format!(
                "GPU declined the scene ({:.1}M triangles exceed one buffer binding) — nothing drawn",
                tris as f32 / 1e6
            ),
        });
        // G / R / S under way: its read-out first, as Blender's header shows it.
        let readout = crate::v2::scene::xform::BTransformModal::now(ctx.state).map(|modal| modal.readout(ctx.state));
        div()
            .flex()
            .flex_col()
            .gap(px(ctx.style.spacing.gap_xs))
            .text_color(to_hsla(ctx.style.colors.text))
            .text_size(px(ctx.style.typography.caption))
            .when_some(readout, |element, line| element.child(div().id("view3d-transform-readout").debug_selector(|| "view3d-transform-readout".to_string()).child(SharedString::from(line))))
            // Blender's wording, in two shades: what the line is about
            // recedes, what it says carries (`view::draw::pair`).
            .when(shown.text, |element| {
                element
                    .child(SharedString::from(self.view_name))
                    .child(pair::phrase(&format!("({scene}) Collection |"), object, ctx))
                    .child(pair::pair("Frame", frame, ctx))
            })
            .when(shown.stats, |element| element.child(pair::strip(&stats, ctx)))
            .when_some(not_drawn, |element, why| {
                element.child(
                    div()
                        .text_color(to_hsla(ctx.style.colors.accent))
                        .child(SharedString::from(why)),
                )
            })
            .into_any_element()
    }
}
