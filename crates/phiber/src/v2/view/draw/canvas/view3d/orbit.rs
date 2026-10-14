//! Viewport navigation drag: orbit, and with Shift a pan, with Ctrl a zoom —
//! Blender's middle-mouse scheme on the drag that orbits here (`scene::navigate`).

use super::super::super::ctx::BRenderCtx;
use crate::v2::scene::navigate::{BDragNav, BNavBase, drag_nav, drag_sets};
use crate::v2::scene::{view_pan, view_pitch, view_yaw, view_zoom};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{Context, Div, IntoElement, Pixels, Point, Render, Window, div};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
struct OrbitDrag {
    /// Picked from the modifiers held as the drag began.
    nav: BDragNav,
    /// The view the drag started from.
    base: BNavBase,
    start: (f32, f32),
}

struct BGhost;

impl Render for BGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// The navigation drag: an orbit, or with Shift held a pan, with Ctrl a zoom
/// (`scene::navigate::drag_nav`). `leave` is set while the canvas looks
/// through the camera: the actions that take the camera's view into the orbit
/// and turn the camera view off (`BCameraView::leave`), applied as the drag
/// starts — Blender's `ED_view3d_persp_switch_from_camera`, which a rotate,
/// a move and a zoom all do.
pub(super) struct BOrbit {
    pub leave: Option<Vec<BAction>>,
    /// The camera frame in canvas pixels, when looking through the camera
    /// (the render-region gesture's reference).
    pub frame: Option<crate::v2::scene::camera_view::BCameraFrame>,
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
        // Ctrl B armed the render region (`render_border`).
        if super::render_border::armed(ctx.state) {
            return super::render_border::surface(ctx, hit, self.frame);
        }
        if crate::v2::scene::circlesel::BCircle::armed(ctx.state) {
            return super::circlesel::surface(ctx, hit);
        }
        // Leaving the camera view, the drag starts from the camera's yaw and
        // pitch — the first two actions `leave` sets — not the orbit's old ones.
        let (yaw, pitch) = match self.leave.as_deref() {
            Some([BAction::Set(_, yaw), BAction::Set(_, pitch), ..]) => {
                (yaw.parse().unwrap_or_else(|_| view_yaw(ctx.state)), pitch.parse().unwrap_or_else(|_| view_pitch(ctx.state)))
            }
            _ => (view_yaw(ctx.state), view_pitch(ctx.state)),
        };
        let base = BNavBase { yaw, pitch, pan: view_pan(ctx.state), zoom: view_zoom(ctx.state) };
        let drag = Arc::new(Mutex::new(OrbitDrag { nav: BDragNav::Orbit, base, start: (0.0, 0.0) }));
        let handler = ctx.handler.clone();
        let start_handler = ctx.handler.clone();
        let leave = self.leave.clone();
        // gpui hands the drag's start as an offset into the surface — the
        // canvas — so the pointer is measured from the canvas too.
        let canvas = hit.bounds.clone();
        let layer = super::hit::layer("view3d-orbit", gpui::CursorStyle::OpenHand, ctx);
        super::lasso::wire(layer, ctx.state, ctx.handler.clone(), hit.clone())
            .on_mouse_down(gpui::MouseButton::Right, hit.clone().on_cursor(ctx))
            .on_click(hit.on_click(ctx))
            .on_drag(
                drag.clone(),
                move |state, position: Point<Pixels>, window, cx| {
                    if let Ok(mut state) = state.lock() {
                        state.start = (f32::from(position.x), f32::from(position.y));
                        // The modifiers held as the drag begins pick the
                        // gesture, as Blender's keymap reads them on the press.
                        let m = window.modifiers();
                        state.nav = drag_nav(m.shift, m.control);
                    }
                    for action in leave.iter().flatten() {
                        (Rc::clone(&start_handler))(action, window, cx);
                    }
                    cx.new(|_| BGhost)
                },
            )
            .on_drag_move::<Arc<Mutex<OrbitDrag>>>(move |event, window, app| {
                let state = event.drag(app);
                let OrbitDrag { nav, base, start } = *state.lock().unwrap();
                // The travel since the press, from the view the drag began on:
                // an orbit turns on the turntable, a pan follows the pointer a
                // pixel for a pixel, a zoom goes up-is-in (`drag_sets`).
                let origin = canvas.get().map(|b| b.origin).unwrap_or_default();
                let at = window.mouse_position() - origin;
                let total = (f32::from(at.x) - start.0, f32::from(at.y) - start.1);
                let width = crate::v2::scene::camera::last_size().0;
                for (key, value) in drag_sets(nav, base, total, width) {
                    (Rc::clone(&handler))(&BAction::Set(key, value), window, app);
                }
            })
    }
}
