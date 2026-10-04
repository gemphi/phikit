//! The render-region gesture surface — what the orbit surface becomes once
//! Ctrl B has armed `view3d.render_border` (`View3dKey::RenderBorder`): a
//! press starts the box, the drag writes its rectangle (`ctx.view3d.box.
//! rect`, painted by `boxsel::paint` as `WM_gesture_draw` draws it), and
//! the release sets the region. In the camera view the box is taken as
//! fractions of the camera frame (`BRenderBorder::from_frame_box`,
//! `render_border_exec`); outside it the box is the viewport's own region
//! as fractions of the region (`BViewBorder::from_region_box`, the other
//! branch of `render_border_exec`).

use super::super::super::ctx::BRenderCtx;
use super::hit::BHit;
use crate::v2::scene::camera_view::BCameraFrame;
use crate::v2::scene::render_border::BRenderBorder;
use crate::v2::scene::view_border::BViewBorder;
use crate::v2::state::{BKey as _, OpKey, View3dKey};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{Context, Div, IntoElement, Pixels, Point, Render, Window};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

struct BBorderDrag {
    start: (f32, f32),
}

struct BGhost;

impl Render for BGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::div()
    }
}

/// The armed surface over the canvas; `frame` is the camera frame in
/// canvas pixels when the view looks through the camera.
pub(super) fn surface(ctx: &BRenderCtx, hit: BHit, frame: Option<BCameraFrame>) -> gpui::Stateful<Div> {
    let drag = Arc::new(Mutex::new(BBorderDrag { start: (0.0, 0.0) }));
    let move_handler = ctx.handler.clone();
    let drop_handler = ctx.handler.clone();
    let hit = Rc::new(hit);
    let move_hit = Rc::clone(&hit);
    super::hit::layer("view3d-render-border", gpui::CursorStyle::Crosshair, ctx)
        .on_drag(drag, move |state, offset: Point<Pixels>, _window, cx| {
            if let Ok(mut state) = state.lock() {
                state.start = (f32::from(offset.x), f32::from(offset.y));
            }
            cx.new(|_| BGhost)
        })
        .on_drag_move::<Arc<Mutex<BBorderDrag>>>(move |event, window, app| {
            let Some(bounds) = move_hit.bounds.get() else { return };
            let start = event.drag(app).lock().map(|s| s.start).unwrap_or((0.0, 0.0));
            let now = window.mouse_position();
            let (x, y) = (f32::from(now.x - bounds.origin.x), f32::from(now.y - bounds.origin.y));
            (Rc::clone(&move_handler))(&BAction::Set(View3dKey::BoxRect.path(), crate::v2::scene::boxsel::encode(start.0, start.1, x, y)), window, app);
        })
        .on_drop::<Arc<Mutex<BBorderDrag>>>(move |state, window, app| {
            let Some(bounds) = hit.bounds.get() else { return };
            let start = state.lock().map(|s| s.start).unwrap_or((0.0, 0.0));
            let now = window.mouse_position();
            let end = (f32::from(now.x - bounds.origin.x), f32::from(now.y - bounds.origin.y));
            let value = match frame {
                Some(f) => match BRenderBorder::from_frame_box((start.0, start.1, end.0, end.1), (f.left, f.top, f.right, f.bottom)) {
                    Some(rect) => format!("set:{}", BRenderBorder::encode(rect)),
                    None => "off".to_string(),
                },
                None => match BViewBorder::from_region_box((start.0, start.1, end.0, end.1), (f32::from(bounds.size.width), f32::from(bounds.size.height))) {
                    Some(rect) => format!("view:{}", BRenderBorder::encode(rect)),
                    None => "view_off".to_string(),
                },
            };
            (Rc::clone(&drop_handler))(&BAction::Set(View3dKey::RenderBorder.path(), String::new()), window, app);
            (Rc::clone(&drop_handler))(&BAction::Set(View3dKey::BoxRect.path(), String::new()), window, app);
            (Rc::clone(&drop_handler))(&BAction::Set(OpKey::RenderBorder.path(), value), window, app);
        })
}

/// Whether Ctrl B has armed the gesture.
pub fn armed(s: &crate::v2::state::BState) -> bool {
    s.get(View3dKey::RenderBorder) == Some(crate::v2::state::val::V_TRUE)
}
