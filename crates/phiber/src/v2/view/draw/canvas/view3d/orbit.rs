//! Viewport orbit interaction surface.

use super::super::super::ctx::BRenderCtx;
use crate::v2::scene::{view_yaw, yaw_key};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{Context, Div, IntoElement, Pixels, Point, Render, Window, div};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
struct OrbitDrag {
    start_yaw: f32,
    start_x: f32,
}

struct BGhost;

impl Render for BGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// The orbit gesture. `leave` is set while the canvas looks through the
/// camera: the actions that take the camera's view into the orbit and turn
/// the camera view off (`BCameraView::leave`), applied as the drag starts —
/// Blender's `ED_view3d_persp_switch_from_camera` on the first orbit.
pub(super) struct BOrbit {
    pub leave: Option<Vec<BAction>>,
}

impl BOrbit {
    /// The gesture surface. A press that moves is the orbit; one that does
    /// not is a click, which `hit` turns into a pick (`hit.rs`) — gpui's
    /// drag threshold is what tells the two apart.
    pub fn render(&self, ctx: &BRenderCtx, hit: super::hit::BHit) -> gpui::Stateful<Div> {
        // G / R / S under way: the pointer drives the transform (`xform`).
        if let Some(modal) = crate::v2::scene::xform::BTransformModal::now(ctx.state) {
            return super::xform::surface(ctx, hit, modal);
        }
        // B armed the box: the drag is the box, not an orbit (`boxsel`).
        if crate::v2::scene::boxsel::armed(ctx.state) {
            return super::boxsel::surface(ctx, hit);
        }
        if crate::v2::scene::circlesel::BCircle::armed(ctx.state) {
            return super::circlesel::surface(ctx, hit);
        }
        // Leaving the camera view, the drag starts from the camera's yaw —
        // the first action `leave` sets — not the orbit's old one.
        let start_yaw = match self.leave.as_deref() {
            Some([BAction::Set(_, yaw), ..]) => yaw.parse().unwrap_or_else(|_| view_yaw(ctx.state)),
            _ => view_yaw(ctx.state),
        };
        let drag = Arc::new(Mutex::new(OrbitDrag { start_yaw, start_x: 0.0 }));
        let handler = ctx.handler.clone();
        let start_handler = ctx.handler.clone();
        let leave = self.leave.clone();
        // This viewport's own yaw (`scene::view_slot`), taken while it draws.
        let yaw = yaw_key();
        let layer = super::hit::layer("view3d-orbit", gpui::CursorStyle::OpenHand);
        super::lasso::wire(layer, ctx.state, ctx.handler.clone(), hit.clone())
            .on_click(hit.on_click(ctx))
            .on_drag(
                drag.clone(),
                move |state, position: Point<Pixels>, window, cx| {
                    if let Ok(mut state) = state.lock() {
                        state.start_x = f32::from(position.x);
                    }
                    for action in leave.iter().flatten() {
                        (Rc::clone(&start_handler))(action, window, cx);
                    }
                    cx.new(|_| BGhost)
                },
            )
            .on_drag_move::<Arc<Mutex<OrbitDrag>>>(move |event, window, app| {
                let state = event.drag(app);
                let (base, start_x) = {
                    let state = state.lock().unwrap();
                    (state.start_yaw, state.start_x)
                };
                // 0.5 degrees per pixel is input mapping, not visual styling.
                let delta = (f32::from(window.mouse_position().x) - start_x) * 0.5;
                let next = (base + delta).rem_euclid(360.0);
                (Rc::clone(&handler))(&BAction::Set(yaw.clone(), next.to_string()), window, app);
            })
    }
}
