//! Viewport orbit interaction surface.

use super::super::super::super::ctx::BRenderCtx;
use crate::v2::scene::{view_yaw, yaw_key};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{Context, Div, IntoElement, Pixels, Point, Render, SharedString, Window, div};
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

pub(super) struct BOrbit;

impl BOrbit {
    pub fn render(&self, c: &BRenderCtx) -> gpui::Stateful<Div> {
        let drag = Arc::new(Mutex::new(OrbitDrag {
            start_yaw: view_yaw(c.s),
            start_x: 0.0,
        }));
        let handler = c.h.clone();
        div()
            .id(SharedString::from("view3d-orbit"))
            .absolute()
            .inset_0()
            .cursor(gpui::CursorStyle::OpenHand)
            .on_drag(
                drag.clone(),
                move |state, position: Point<Pixels>, _, cx| {
                    if let Ok(mut state) = state.lock() {
                        state.start_x = f32::from(position.x);
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
                (Rc::clone(&handler))(&BAction::Set(yaw_key(), next.to_string()), window, app);
            })
    }
}
