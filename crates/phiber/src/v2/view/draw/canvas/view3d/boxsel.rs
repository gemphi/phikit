//! The box-select gesture surface — what the orbit surface becomes once B
//! has armed `view3d.select_box` (`scene::boxsel::armed`): a press starts
//! the box, the drag writes its rectangle (`ctx.view3d.box.rect`, painted
//! by [`paint`] as `WM_gesture_draw` draws it: a dashed outline), and the
//! release fires the selection through `ctx.op.select`. Shift adds, Ctrl
//! subtracts (`BBoxMode`).

use super::super::super::ctx::BRenderCtx;
use super::hit::BHit;
use crate::v2::scene::boxsel::{BBox2, BBoxMode, boxed_names, encode};
use crate::v2::scene::pick::BPickView;
use crate::v2::scene::pick::occlude::BPickScene;
use crate::v2::state::{BKey as _, OpKey, View3dKey};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{BorderStyle, Bounds, Context, Div, Hsla, IntoElement, Pixels, Point, Render, Window, div, outline, point, px, size};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Where the press landed, in canvas pixels.
struct BBoxDrag {
    start: (f32, f32),
}

struct BGhost;

impl Render for BGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// The armed surface over the canvas.
pub(super) fn surface(ctx: &BRenderCtx, hit: BHit) -> gpui::Stateful<Div> {
    let drag = Arc::new(Mutex::new(BBoxDrag { start: (0.0, 0.0) }));
    let xray = crate::v2::scene::BXray::now(ctx.state).active(crate::v2::scene::view_shading(ctx.state));
    let move_handler = ctx.handler.clone();
    let drop_handler = ctx.handler.clone();
    let hit = Rc::new(hit);
    let start_hit = Rc::clone(&hit);
    let move_hit = Rc::clone(&hit);
    super::hit::layer("view3d-box-select", gpui::CursorStyle::Crosshair, ctx)
        .on_drag(drag, move |state, offset: Point<Pixels>, _window, cx| {
            // gpui hands the start as an offset from the surface, which is
            // the canvas: already canvas pixels.
            let _ = &start_hit;
            if let Ok(mut state) = state.lock() {
                state.start = (f32::from(offset.x), f32::from(offset.y));
            }
            cx.new(|_| BGhost)
        })
        .on_drag_move::<Arc<Mutex<BBoxDrag>>>(move |event, window, app| {
            let Some(bounds) = move_hit.bounds.get() else { return };
            let start = event.drag(app).lock().map(|s| s.start).unwrap_or((0.0, 0.0));
            let now = window.mouse_position();
            let (x, y) = (f32::from(now.x - bounds.origin.x), f32::from(now.y - bounds.origin.y));
            (Rc::clone(&move_handler))(&BAction::Set(View3dKey::BoxRect.path(), encode(start.0, start.1, x, y)), window, app);
        })
        .on_drop::<Arc<Mutex<BBoxDrag>>>(move |state, window, app| {
            let Some(bounds) = hit.bounds.get() else { return };
            let start = state.lock().map(|s| s.start).unwrap_or((0.0, 0.0));
            let now = window.mouse_position();
            let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            let end = (f32::from(now.x - bounds.origin.x), f32::from(now.y - bounds.origin.y));
            let boxed = BBox2 { a: BPickView::canvas_point(start.0, start.1, w, h), b: BPickView::canvas_point(end.0, end.1, w, h) };
            let view = BPickView { aspect: w / h.max(1.0), ..hit.view };
            let allowed = |name: &str| !hit.unselectable.contains(name);
            // What the box overlaps, then — without X-ray — only what a ray
            // through it sees first: an object behind another is not taken.
            let overlapped = boxed_names(&hit.scene, &view, boxed, allowed);
            let mut names = if xray { overlapped } else { BPickScene::new(&hit.scene, allowed).visible(&view, boxed, overlapped) };
            // Cameras, lights and empties by their origin, as Blender's overlay draws them over everything.
            let inside = |x: f32, y: f32| boxed.holds(BPickView::canvas_point(x, y, w, h));
            names.extend(crate::v2::scene::pick::extra::BPickExtra::origins_where(&hit.extras, &view, (w, h), inside));
            let modifiers = window.modifiers();
            let op = format!("{}{}", BBoxMode::of(modifiers.shift, modifiers.control).op(), names.join(","));
            (Rc::clone(&drop_handler))(&BAction::Set(OpKey::Select.path(), op), window, app);
        })
}

/// `wm_gesture_draw_rect`: the box as it is dragged, a dashed outline in
/// the overlay ink over a solid one in the canvas shade, so it reads on
/// any surface.
pub(super) fn paint(bounds: Bounds<Pixels>, rect: (f32, f32, f32, f32), back: Hsla, ink: Hsla, window: &mut Window) {
    let (x0, y0, x1, y1) = rect;
    let at = Bounds::new(
        point(bounds.origin.x + px(x0.min(x1)), bounds.origin.y + px(y0.min(y1))),
        size(px((x1 - x0).abs()), px((y1 - y0).abs())),
    );
    window.paint_quad(outline(at, back, BorderStyle::Solid));
    window.paint_quad(outline(at, ink, BorderStyle::Dashed));
}
