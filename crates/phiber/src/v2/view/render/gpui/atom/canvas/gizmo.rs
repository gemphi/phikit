//! View3D navigation gizmo preserving existing yaw/zoom actions.

pub mod compass;

use super::super::super::{color::to_hsla, ctx::BRenderCtx};
use crate::v2::scene::{
    pan_x_key, pitch_key, pitch_step, view_pitch, view_yaw, view_zoom, yaw_key, zoom_key,
    zoom_step,
};
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

pub struct BGizmo;

impl BGizmo {
    fn control(
        &self,
        name: &'static str,
        tip: &'static str,
        action: BAction,
        c: &BRenderCtx,
    ) -> gpui::Stateful<gpui::Div> {
        let handler = c.h.clone();
        let dispatched = action.clone();
        let selector = format!("gizmo-{name}");
        div()
            .id(SharedString::from(format!("gizmo-{name}")))
            .debug_selector(move || selector.clone())
            .flex()
            .items_center()
            .justify_center()
            .w(px(c.style.dimensions.gizmo_button_size))
            .h(px(c.style.dimensions.gizmo_button_size))
            .rounded(px(c.style.radius_for(BStyleRole::Button)))
            .cursor_pointer()
            .hover(|style| style.bg(to_hsla(c.style.colors.bg_hover)))
            .child(
                svg()
                    .path(format!("icons/{name}.svg"))
                    .size(px(c.style.dimensions.gizmo_icon_size))
                    .flex_shrink_0()
                    .text_color(to_hsla(c.style.colors.text)),
            )
            .on_click(move |_e: &ClickEvent, w, cx| {
                let _ = tip;
                (Rc::clone(&handler))(&dispatched, w, cx);
            })
    }

    fn yaw_step(&self, c: &BRenderCtx, delta: f32) -> BAction {
        BAction::Set(
            yaw_key(),
            format!("{}", (view_yaw(c.s) + delta).rem_euclid(360.0)),
        )
    }

    fn pitch_step_action(&self, c: &BRenderCtx, delta: f32) -> BAction {
        BAction::Set(pitch_key(), format!("{}", pitch_step(view_pitch(c.s), delta)))
    }

    fn zoom_action(&self, c: &BRenderCtx, factor: f32) -> BAction {
        BAction::Set(zoom_key(), format!("{}", zoom_step(view_zoom(c.s), factor)))
    }

    fn surface(&self, c: &BRenderCtx) -> gpui::Div {
        let mut border = c.style.colors.text;
        border.3 = c.style.effects.viewport_grid_alpha;
        div()
            .border(px(c.style.strokes.border))
            .border_color(to_hsla(border))
            .bg(to_hsla(c.style.colors.bg_modal_dim))
            .rounded(px(c.style.radius_for(BStyleRole::Card)))
    }

    fn pill(&self, c: &BRenderCtx, children: Vec<gpui::Stateful<gpui::Div>>) -> gpui::Div {
        let mut column = self
            .surface(c)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(c.style.spacing.gap_xs / 2.0))
            .px(px(c.style.spacing.pad_xs / 2.0))
            .py(px(c.style.spacing.pad_xs * 0.75));
        for child in children {
            column = column.child(child);
        }
        column
    }

    fn orbit(&self, c: &BRenderCtx) -> gpui::Div {
        // Blender's navigate gizmo has four orbit arrows around the compass
        // (up / left / right / down). Lay them as a 2-column grid so the
        // vertical pair flanks the compass and the horizontal pair is below.
        self.surface(c)
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(c.style.spacing.gap_xs / 2.0))
            .px(px(c.style.spacing.pad_xs * 0.75))
            .py(px(c.style.spacing.pad_xs / 2.0))
            .child(self.control("arrow_left", "Orbit Left", self.yaw_step(c, -15.0), c))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(c.style.spacing.gap_xs / 2.0))
                    .child(self.control(
                        "arrow_up",
                        "Orbit Up",
                        self.pitch_step_action(c, 15.0),
                        c,
                    ))
                    .child(self.control(
                        "arrow_down",
                        "Orbit Down",
                        self.pitch_step_action(c, -15.0),
                        c,
                    )),
            )
            .child(self.control("arrow_right", "Orbit Right", self.yaw_step(c, 15.0), c))
    }

    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let persp = crate::v2::scene::is_perspective(c.s);
        div()
            .flex()
            .flex_col()
            .items_end()
            .gap(px(c.style.spacing.gap_sm))
            .child(compass::BCompass.render(c))
            .child(self.orbit(c))
            .child(self.pill(
                c,
                vec![
                    self.control("zoom_in", "Zoom In", self.zoom_action(c, 1.2), c),
                    self.control("zoom_out", "Zoom Out", self.zoom_action(c, 1.0 / 1.2), c),
                    // Recenter pan: click "grab" resets horizontal pan to 0.
                    // Vertical follows Frame All. Data-driven single-click semantics.
                    self.control(
                        "grab",
                        "Recenter View",
                        BAction::Reset(pan_x_key()),
                        c,
                    ),
                    // Frame All: reset zoom to 1.0. Kept as a single-key reset so
                    // the button dispatches a single BAction. Pitch/pan resets are
                    // wired to the arrow / grab controls above.
                    self.control(
                        "home",
                        "Frame All",
                        BAction::Set(zoom_key(), "1".to_string()),
                        c,
                    ),
                    // Perspective / Orthographic toggle — matches Blender's numpad-5 shortcut.
                    self.control(
                        if persp { "perspective" } else { "orthographic" },
                        if persp { "Switch to Orthographic" } else { "Switch to Perspective" },
                        BAction::Set(
                            crate::v2::scene::perspective_key(),
                            if persp { "false".to_string() } else { "true".to_string() },
                        ),
                        c,
                    ),
                ],
            ))
            .into_any_element()
    }
}
