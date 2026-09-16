//! View3D canvas composition; interaction and mesh painting live in focused helpers.

mod hit;
mod mesh;
mod orbit;

use self::hit::BHit;
use self::mesh::{BMesh, BMeshPalette};
use self::orbit::BOrbit;
use super::super::super::{color::to_hsla, ctx::BRenderCtx};
use crate::v2::scene::{
    project3_full, view_pan, view_pitch, view_yaw, view_zoom, zoom_key, zoom_step,
};
use crate::v2::state::key::K_OBJECT;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{
    AnyElement, Div, Pixels, ScrollDelta, ScrollWheelEvent, SharedString, canvas, div, px, svg,
};
use std::rc::Rc;

pub struct BView3d;

impl BView3d {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let grid = c.style.viewport_grid_stroke();
        let axis_x = c.style.axis_stroke(c.style.colors.axis_x);
        let axis_y = c.style.axis_stroke(c.style.colors.axis_y);
        let line = px(grid.width);
        let vp = to_hsla(c.style.viewport_surface());
        let zoom_handler = c.h.clone();
        let current_zoom = view_zoom(c.s);
        let mut element = div()
            .id(SharedString::from("view3d-canvas"))
            .debug_selector(|| "view3d-canvas".to_string())
            .flex_1()
            .min_h_0()
            .size_full()
            .relative()
            .overflow_hidden()
            .bg(vp)
            .on_scroll_wheel(move |event: &ScrollWheelEvent, window, app| {
                // Wheel thresholds and factors are input mechanics, not UI design.
                let delta = match event.delta {
                    ScrollDelta::Pixels(point) => f32::from(point.y),
                    ScrollDelta::Lines(point) => point.y * 20.0,
                };
                if delta.abs() < 0.5 {
                    return;
                }
                let factor = if delta > 0.0 { 1.1 } else { 1.0 / 1.1 };
                let next = zoom_step(current_zoom, factor);
                (Rc::clone(&zoom_handler))(
                    &BAction::Set(zoom_key(), next.to_string()),
                    window,
                    app,
                );
            });

        // Fractions are normalized scene coordinates and remain GPUI mechanics.
        for fraction in [0.125_f32, 0.25, 0.375, 0.625, 0.75, 0.875] {
            element = element
                .child(vertical(fraction, to_hsla(grid.color), line))
                .child(horizontal(fraction, to_hsla(grid.color), line));
        }
        element = element
            .child(horizontal(0.5, to_hsla(axis_x.color), line))
            .child(vertical(0.5, to_hsla(axis_y.color), line))
            .child(BOrbit.render(c))
            .child(
                div()
                    .absolute()
                    .left(gpui::relative(0.16))
                    .top(gpui::relative(0.31))
                    .child(
                        svg()
                            .path("icons/camera.svg")
                            .size(px(c.style.dimensions.object_icon_size))
                            .text_color(to_hsla(c.style.colors.text_dim)),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .left(gpui::relative(0.76))
                    .top(gpui::relative(0.25))
                    .child(
                        svg()
                            .path("icons/light.svg")
                            .size(px(c.style.dimensions.object_glyph_size))
                            .text_color(to_hsla(c.style.colors.text_dim)),
                    ),
            );

        let yaw = view_yaw(c.s);
        let pitch = view_pitch(c.s);
        let pan = view_pan(c.s);
        let zoom = view_zoom(c.s);
        let active = c.s.get(&K_OBJECT).unwrap_or("Cube").to_string();
        let panel = c.style.colors.bg_panel;
        let palette = BMeshPalette {
            cube: to_hsla(c.style.shade(panel, c.style.effects.node_grid_alpha)),
            sphere: to_hsla(c.style.shade(
                panel,
                c.style.effects.node_grid_alpha + c.style.effects.sheen_lift,
            )),
            gem: to_hsla(c.style.colors.accent_soft),
            loaded: to_hsla(c.style.colors.bg_active),
            accent: to_hsla(c.style.colors.accent),
        };
        let scene: Vec<_> = crate::v2::scene::scene_source(c.s)
            .into_iter()
            .filter(|object| {
                !crate::v2::part::custom::outliner::tree::is_hidden(c.s, mesh::name(object))
            })
            .collect();
        let mut hits = Vec::new();
        for object in &scene {
            let (x, y) = mesh::ground(object);
            let (screen_x, screen_y, _) =
                project3_full(x, 0.0, y, yaw, pitch, pan, zoom);
            hits.push((screen_x, screen_y, mesh::name(object).to_string()));
        }
        hits.sort_by(|left, right| {
            right
                .1
                .partial_cmp(&left.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut meshes: Vec<_> = scene
            .iter()
            .map(|object| {
                BMesh::build(
                    object,
                    yaw,
                    pitch,
                    pan,
                    zoom,
                    palette,
                    active == mesh::name(object),
                )
            })
            .collect();
        meshes.sort_by(|left, right| {
            right
                .depth
                .partial_cmp(&left.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let outline_width = c.style.strokes.border * 2.0;
        element = element.child(
            canvas(
                |_, _, _| {},
                move |bounds, _, window, _| BMesh::paint(bounds, &meshes, outline_width, window),
            )
            .absolute()
            .inset_0()
            .size_full(),
        );
        for (x, y, name) in hits {
            element = element.child(BHit { x, y, name }.render(c));
        }
        element
            .child(
                div()
                    .absolute()
                    .top(px(c.style.spacing.pad_md))
                    .left(px(c.style.spacing.pad_md))
                    .child(super::overlay::BOverlay.render(c)),
            )
            .child(
                div()
                    .absolute()
                    .top(px(c.style.spacing.pad_md))
                    .right(px(c.style.spacing.pad_md))
                    .child(super::gizmo::BGizmo.render(c)),
            )
            .into_any_element()
    }
}

fn vertical(fraction: f32, color: gpui::Hsla, width: Pixels) -> Div {
    div()
        .absolute()
        .top_0()
        .h_full()
        .w(width)
        .left(gpui::relative(fraction))
        .bg(color)
}

fn horizontal(fraction: f32, color: gpui::Hsla, width: Pixels) -> Div {
    div()
        .absolute()
        .left_0()
        .w_full()
        .h(width)
        .top(gpui::relative(fraction))
        .bg(color)
}

use crate::v2::view::render::gpui::prim::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BView3d {
    fn render(&self, c: &crate::v2::view::render::gpui::ctx::BRenderCtx) -> gpui::AnyElement {
        BView3d::render(self, c)
    }
}
