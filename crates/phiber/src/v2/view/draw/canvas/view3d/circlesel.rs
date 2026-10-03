//! The circle-select surface — what the orbit surface becomes while C has
//! the tool on (`scene::circlesel`): the pointer moves the brush
//! (`ctx.view3d.circle.at`), the left button paints a selection, Shift or
//! the middle button paints it away, the wheel resizes the brush and the
//! right button ends the tool. [`paint`] draws the brush as
//! `wm_gesture_draw_circle` does: a dashed ring over a solid one.

use super::super::super::ctx::BRenderCtx;
use super::hit::BHit;
use crate::v2::scene::boxsel::BBoxMode;
use crate::v2::scene::circlesel::BCircle;
use crate::v2::scene::pick::BPickView;
use crate::v2::scene::select::BSelection;
use crate::v2::state::{BKey as _, OpKey, View3dKey, val::V_FALSE};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{App, BorderStyle, Bounds, Div, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Point, ScrollDelta, ScrollWheelEvent, Window, outline, point, px, size};
use std::rc::Rc;

type BHandler = Rc<dyn Fn(&BAction, &mut Window, &mut App)>;

/// What one brush dab needs: the scene as hit-tested, the selection and the
/// brush as this frame drew them.
struct BBrush {
    hit: BHit,
    selection: BSelection,
    radius: f32,
    handler: BHandler,
}

impl BBrush {
    /// The pointer in canvas pixels, and the canvas size.
    fn local(&self, at: Point<Pixels>) -> Option<(f32, f32, f32, f32)> {
        let bounds = self.hit.bounds.get()?;
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        Some((f32::from(at.x - bounds.origin.x), f32::from(at.y - bounds.origin.y), w, h))
    }

    fn fire(&self, action: BAction, window: &mut Window, app: &mut App) {
        (self.handler)(&action, window, app);
    }

    /// Move the brush, and paint with it when a button is down.
    fn dab(&self, at: Point<Pixels>, paint: Option<bool>, window: &mut Window, app: &mut App) {
        let Some((x, y, w, h)) = self.local(at) else { return };
        self.fire(BAction::Set(View3dKey::CircleAt.path(), BCircle::encode_at(x, y)), window, app);
        let Some(deselect) = paint else { return };
        let view = BPickView { aspect: w / h.max(1.0), ..self.hit.view };
        let circle = BCircle { x, y, radius: self.radius };
        let mut names = circle.names(&self.hit.scene, &view, w, h, |n| !self.hit.unselectable.contains(n));
        let under = |px: f32, py: f32| (px - circle.x).powi(2) + (py - circle.y).powi(2) <= circle.radius.powi(2);
        names.extend(crate::v2::scene::pick::extra::BPickExtra::origins_where(&self.hit.extras, &view, (w, h), under));
        let mode = if deselect { BBoxMode::Sub } else { BBoxMode::Add };
        if let Some(op) = mode.fire(names, |n| self.selection.contains(n)) {
            self.fire(BAction::Set(OpKey::Select.path(), op), window, app);
        }
    }
}

/// Which way a held button paints: `Some(true)` deselects.
fn painting(button: Option<MouseButton>, shift: bool) -> Option<bool> {
    match button {
        Some(MouseButton::Left) => Some(shift),
        Some(MouseButton::Middle) => Some(true),
        _ => None,
    }
}

/// The surface over the canvas while the tool is on.
pub(super) fn surface(ctx: &BRenderCtx, hit: BHit) -> gpui::Stateful<Div> {
    let brush = Rc::new(BBrush { hit, selection: BSelection::now(ctx.state), radius: BCircle::radius(ctx.state), handler: ctx.handler.clone() });
    let (on_move, on_down, on_wheel) = (Rc::clone(&brush), Rc::clone(&brush), Rc::clone(&brush));
    super::hit::layer("view3d-circle-select", gpui::CursorStyle::Crosshair, ctx)
        .on_mouse_move(move |e: &MouseMoveEvent, window, app| on_move.dab(e.position, painting(e.pressed_button, e.modifiers.shift), window, app))
        .on_any_mouse_down(move |e: &MouseDownEvent, window, app| match e.button {
            MouseButton::Right => on_down.fire(BAction::Set(View3dKey::CircleSelect.path(), V_FALSE.to_string()), window, app),
            button => on_down.dab(e.position, painting(Some(button), e.modifiers.shift), window, app),
        })
        .on_scroll_wheel(move |e: &ScrollWheelEvent, window, app| {
            // Wheel up shrinks the brush, down grows it (`GESTURE_MODAL_CIRCLE_SUB / _ADD`).
            let dy = match e.delta {
                ScrollDelta::Lines(lines) => lines.y,
                ScrollDelta::Pixels(pixels) => f32::from(pixels.y),
            };
            let grow = dy < 0.0;
            let radius = BCircle::resized(on_wheel.radius, grow);
            on_wheel.fire(BAction::Set(View3dKey::CircleRadius.path(), format!("{radius:.1}")), window, app);
        })
}

/// The brush ring, in the box's two inks.
pub(super) fn paint(bounds: Bounds<Pixels>, circle: BCircle, back: Hsla, ink: Hsla, window: &mut Window) {
    let (r, across) = (circle.radius, circle.radius + circle.radius);
    let at = Bounds::new(point(bounds.origin.x + px(circle.x - r), bounds.origin.y + px(circle.y - r)), size(px(across), px(across)));
    window.paint_quad(outline(at, back, BorderStyle::Solid).corner_radii(px(r)));
    window.paint_quad(outline(at, ink, BorderStyle::Dashed).corner_radii(px(r)));
}
