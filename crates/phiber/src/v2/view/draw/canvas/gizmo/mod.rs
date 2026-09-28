//! View3D navigation gizmo preserving existing yaw/zoom actions.

pub mod compass;

use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
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
        ctx: &BRenderCtx,
    ) -> gpui::Stateful<gpui::Div> {
        let handler = ctx.handler.clone();
        let dispatched = action.clone();
        let selector = format!("gizmo-{name}");
        div()
            .id(SharedString::from(format!("gizmo-{name}")))
            .debug_selector(move || selector.clone())
            .flex()
            .items_center()
            .justify_center()
            .w(px(ctx.style.dimensions.gizmo_button_size))
            .h(px(ctx.style.dimensions.gizmo_button_size))
            .rounded(px(ctx.style.radius_for(BStyleRole::Button)))
            .cursor_pointer()
            .hover(|style| style.bg(to_hsla(ctx.style.host_item_hover(ctx.host))))
            .child(
                svg()
                    .path(format!("icons/{name}.svg"))
                    .size(px(ctx.style.dimensions.gizmo_icon_size))
                    .flex_shrink_0()
                    .text_color(to_hsla(ctx.style.colors.text)),
            )
            .on_click(move |_e: &ClickEvent, w, cx| {
                let _ = tip;
                (Rc::clone(&handler))(&dispatched, w, cx);
            })
    }

    fn yaw_step(&self, ctx: &BRenderCtx, delta: f32) -> BAction {
        BAction::Set(
            yaw_key(),
            format!("{}", (view_yaw(ctx.state) + delta).rem_euclid(360.0)),
        )
    }

    fn pitch_step_action(&self, ctx: &BRenderCtx, delta: f32) -> BAction {
        BAction::Set(pitch_key(), format!("{}", pitch_step(view_pitch(ctx.state), delta)))
    }

    fn zoom_action(&self, ctx: &BRenderCtx, factor: f32) -> BAction {
        BAction::Set(zoom_key(), format!("{}", zoom_step(view_zoom(ctx.state), factor)))
    }

    fn surface(&self, ctx: &BRenderCtx) -> gpui::Div {
        let mut border = ctx.style.colors.text;
        border.3 = ctx.style.effects.viewport_grid_alpha;
        let surface = div()
            .border(px(ctx.style.strokes.border))
            .border_color(to_hsla(border))
            // The gizmos are floating chrome, so they take the same fill as
            // the toolbar and sidebar beside them. The modal dim they used
            // was far darker than anything else over the canvas.
            .bg(crate::v2::view::draw::pane::floating_bg(ctx));
        // A gizmo sits on the scene, so it casts on it — a rung under a
        // dropdown, as `BElevation::Floating` says.
        crate::v2::view::draw::flex::shadowed(
            surface.rounded(px(ctx.style.radius_for(BStyleRole::Card))),
            ctx.style.shadow_for(crate::v2::theme::BElevation::Floating),
        )
    }

    fn pill(&self, ctx: &BRenderCtx, list: Vec<gpui::Stateful<gpui::Div>>) -> gpui::Div {
        let mut column = self
            .surface(ctx)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(ctx.style.spacing.gap_xs / 2.0))
            .px(px(ctx.style.spacing.pad_xs / 2.0))
            .py(px(ctx.style.spacing.pad_xs * 0.75));
        for child in list {
            column = column.child(child);
        }
        column
    }

    fn orbit(&self, ctx: &BRenderCtx) -> gpui::Div {
        // Blender's navigate gizmo has four orbit arrows around the compass
        // (up / left / right / down). Lay them as a 2-column grid so the
        // vertical pair flanks the compass and the horizontal pair is below.
        self.surface(ctx)
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(ctx.style.spacing.gap_xs / 2.0))
            .px(px(ctx.style.spacing.pad_xs * 0.75))
            .py(px(ctx.style.spacing.pad_xs / 2.0))
            .child(self.control("arrow_left", "Orbit Left", self.yaw_step(ctx, -15.0), ctx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(ctx.style.spacing.gap_xs / 2.0))
                    .child(self.control(
                        "arrow_up",
                        "Orbit Up",
                        self.pitch_step_action(ctx, 15.0),
                        ctx,
                    ))
                    .child(self.control(
                        "arrow_down",
                        "Orbit Down",
                        self.pitch_step_action(ctx, -15.0),
                        ctx,
                    )),
            )
            .child(self.control("arrow_right", "Orbit Right", self.yaw_step(ctx, 15.0), ctx))
    }

    /// The navigate sphere. It lives in the corner the user chose, because it
    /// is the thing they aim at.
    pub fn compass(&self, ctx: &BRenderCtx) -> AnyElement {
        self::compass::BCompass.render(ctx)
    }

    /// The square clusters — orbit arrows and the zoom / view pill. They sit
    /// at the other end of the same side, so the sphere is not buried under a
    /// tall column of boxes.
    pub fn controls(&self, ctx: &BRenderCtx, camera: &crate::v2::scene::camera_view::BCameraSlot) -> AnyElement {
        let persp = crate::v2::scene::is_perspective(ctx.state);
        let mut pill = Vec::with_capacity(6);
        pill.push(self.control("zoom_in", "Zoom In", self.zoom_action(ctx, 1.2), ctx));
        pill.push(self.control("zoom_out", "Zoom Out", self.zoom_action(ctx, 1.0 / 1.2), ctx));
        // Recenter pan: click "grab" resets horizontal pan to 0. Vertical
        // follows Frame All. Data-driven single-click semantics.
        pill.push(self.control("grab", "Recenter View", BAction::Reset(pan_x_key()), ctx));
        // Frame All: reset zoom to 1.0. Kept as a single-key reset so the
        // button dispatches a single BAction.
        pill.push(self.control("home", "Frame All", BAction::Set(zoom_key(), "1".to_string()), ctx));
        // The camera view, in and out (`VIEW3D_OT_view_camera`, Numpad 0),
        // for the area this gizmo sits in — only where there is one.
        if let Some(toggle) = camera.toggle() {
            pill.push(self.control(
                "camera",
                if camera.through { "Leave Camera View" } else { "View Camera" },
                toggle,
                ctx,
            ));
        }
        // Perspective / Orthographic toggle — matches Blender's numpad-5 shortcut.
        pill.push(self.control(
            if persp { "perspective" } else { "orthographic" },
            if persp { "Switch to Orthographic" } else { "Switch to Perspective" },
            BAction::Set(crate::v2::scene::perspective_key(), if persp { "false".to_string() } else { "true".to_string() }),
            ctx,
        ));
        div()
            .flex()
            .flex_col()
            // The groups share a centre line, so each keeps the same margin
            // from the viewport edge. Aligning their right edges made a round
            // group look inset next to a square one.
            .items_center()
            .gap(px(ctx.style.spacing.gap_sm))
            .child(self.orbit(ctx))
            .child(self.pill(ctx, pill))
            .into_any_element()
    }
}
