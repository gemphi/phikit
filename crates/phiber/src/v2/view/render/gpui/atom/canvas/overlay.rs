//! View3D top-left perspective and active-object overlay.
//!
//! Lines rendered here are fed entirely from state + Scene_Source, matching
//! Blender's viewport info line ("User Perspective" / "(Scene) Collection |
//! <object>" / "Frame N" / "Verts: V | Faces: F"). No values are hard-coded.

use super::super::super::{color::to_hsla, ctx::BRenderCtx};
use crate::v2::scene::{scene_metrics, scene_source};
use crate::v2::state::key::{K_FRAME, K_OBJECT, K_SCENE};
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

pub struct BOverlay;

impl BOverlay {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let object = c.s.get(&K_OBJECT).unwrap_or("Cube");
        let scene = c.s.get(&K_SCENE).unwrap_or("Scene");
        let frame = c.s.get(&K_FRAME).unwrap_or("1");
        let (obj_count, verts, faces) = scene_metrics(&scene_source(c.s));
        div()
            .flex()
            .flex_col()
            .gap(px(c.style.spacing.gap_xs))
            .text_color(to_hsla(c.style.colors.text))
            .text_size(px(c.style.typography.caption))
            .child(SharedString::from("User Perspective"))
            .child(SharedString::from(format!(
                "({scene}) Collection | {object}"
            )))
            .child(SharedString::from(format!("Frame {frame}")))
            .child(SharedString::from(format!(
                "Objects: {obj_count} | Verts: {verts} | Faces: {faces}"
            )))
            .into_any_element()
    }
}
