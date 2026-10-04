//! `image.render_border` over the picture — once Ctrl B has armed it
//! (`ImageKey::BorderArm`), a left drag over the Image editor is the render
//! region: the box written as it goes (`ImageKey::BorderRect`, painted as
//! `WM_gesture_draw` draws it) and, on release, taken as fractions of the
//! picture (`BRenderBorder::from_image_box`, snapped to the render frame's
//! pixels) into `ctx.op.render_border`; an empty or whole-frame box turns
//! the region off, as Blender's does.

use super::BImageView;
use crate::phiphx::blend::BlendRenderBorder;
use crate::v2::state::{BKey as _, BState, ImageKey, OpKey, val::V_TRUE};
use crate::v2::view::draw::ctx::BRenderCtx;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{App, Bounds, Pixels, Point, Window, point, px, size};
use std::cell::Cell;
use std::rc::Rc;

/// The gesture over one Image editor's picture.
#[derive(Clone)]
pub struct BBorderGesture {
    pub armed: bool,
    pub start: Rc<Cell<Option<Point<Pixels>>>>,
    pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    pub view: BImageView,
    pub handler: crate::v2::view::draw::ctx::BActionHandler,
    /// The render frame's size, for the snap, and the region as it stands
    /// (a box over a cropped picture is inside the crop).
    pub frame: (u32, u32),
    pub current: BlendRenderBorder,
}

impl BBorderGesture {
    /// The gesture for this frame: armed or not, over `bounds`, with the
    /// render frame's size and the region as it stands.
    pub fn of(ctx: &BRenderCtx, bounds: Rc<Cell<Option<Bounds<Pixels>>>>, view: BImageView) -> Self {
        let (frame, current) = Self::render_frame(ctx.state);
        let armed = ctx.state.get(ImageKey::BorderArm) == Some(V_TRUE);
        Self { armed, start: Rc::new(Cell::new(None)), bounds, view, handler: ctx.handler.clone(), frame, current }
    }

    /// The frame a render would fill (`BKE_render_resolution`), for snapping
    /// a region to its pixels, and the region as it stands.
    fn render_frame(s: &BState) -> ((u32, u32), BlendRenderBorder) {
        let set = crate::v2::scene::render_set(s);
        crate::v2::scene::source::with_loaded(|m| {
            let (frame, file) = m.map(|m| (set.resolution.pixels(m), m.render.border)).unwrap_or_else(|| (set.resolution.pixels(&Default::default()), Default::default()));
            (frame, crate::v2::scene::render_border::BRenderBorder::now(s, file))
        })
    }

    /// The window element with the gesture's handlers and, armed, the cross-hair.
    pub fn attach(self, element: gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> {
        let (down, moved, up) = (self.clone(), self.clone(), self.clone());
        element
            .when(self.armed, |this| this.cursor(gpui::CursorStyle::Crosshair))
            .on_mouse_down(gpui::MouseButton::Left, move |event, _, _| down.press(event.position))
            .on_mouse_move(move |event, window, app| moved.drag(event, window, app))
            .on_mouse_up(gpui::MouseButton::Left, move |event, window, app| up.release(event.position, window, app))
    }

    pub fn press(&self, at: Point<Pixels>) {
        if self.armed {
            self.start.set(Some(at));
        }
    }

    pub fn drag(&self, event: &gpui::MouseMoveEvent, window: &mut Window, app: &mut App) {
        let (Some(from), Some(b)) = (self.start.get(), self.bounds.get()) else { return };
        if event.pressed_button != Some(gpui::MouseButton::Left) {
            self.start.set(None);
            return;
        }
        let (x0, y0) = (f32::from(from.x - b.origin.x), f32::from(from.y - b.origin.y));
        let (x1, y1) = (f32::from(event.position.x - b.origin.x), f32::from(event.position.y - b.origin.y));
        (self.handler)(&BAction::Set(ImageKey::BorderRect.path(), crate::v2::scene::boxsel::encode(x0, y0, x1, y1)), window, app);
    }

    pub fn release(&self, at: Point<Pixels>, window: &mut Window, app: &mut App) {
        let (Some(from), Some(b)) = (self.start.take(), self.bounds.get()) else { return };
        (self.handler)(&BAction::Set(ImageKey::BorderArm.path(), String::new()), window, app);
        (self.handler)(&BAction::Set(ImageKey::BorderRect.path(), String::new()), window, app);
        let Some((w, h)) = self.view.image() else { return };
        let rect = self.view.rect(w as f32, h as f32, b);
        let (rw, rh) = (f32::from(rect.size.width).max(1.0), f32::from(rect.size.height).max(1.0));
        let norm = |p: Point<Pixels>| ((f32::from(p.x - rect.origin.x) / rw).clamp(0.0, 1.0), (f32::from(p.y - rect.origin.y) / rh).clamp(0.0, 1.0));
        let (a, z) = (norm(from), norm(at));
        let value = match crate::v2::scene::render_border::BRenderBorder::from_image_box((a.0, a.1, z.0, z.1), self.current, self.frame.0, self.frame.1) {
            Some(rect) => format!("set:{}", crate::v2::scene::render_border::BRenderBorder::encode(rect)),
            None => "off".to_string(),
        };
        (self.handler)(&BAction::Set(OpKey::RenderBorder.path(), value), window, app);
    }
}

/// `wm_gesture_draw_rect` over the picture: a dashed outline in the overlay
/// ink over a solid one in the canvas shade.
pub fn paint_box(bounds: Bounds<Pixels>, rect: (f32, f32, f32, f32), back: gpui::Hsla, ink: gpui::Hsla, window: &mut Window) {
    let (x0, y0, x1, y1) = rect;
    let at = Bounds::new(point(bounds.origin.x + px(x0.min(x1)), bounds.origin.y + px(y0.min(y1))), size(px((x1 - x0).abs()), px((y1 - y0).abs())));
    window.paint_quad(gpui::outline(at, back, gpui::BorderStyle::Solid));
    window.paint_quad(gpui::outline(at, ink, gpui::BorderStyle::Dashed));
}
