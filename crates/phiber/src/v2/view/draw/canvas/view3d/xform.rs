//! The surface over the canvas while G, R or S is under way
//! (`scene::xform`): the pointer drives the transform in the view it is
//! drawn in, the left button keeps it, the right button puts it back.

use super::super::super::ctx::BRenderCtx;
use super::hit::BHit;
use crate::v2::scene::pick::BPickView;
use crate::v2::scene::xform::BTransformModal;
use crate::v2::state::{BKey as _, OpKey};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{App, Div, MouseButton, MouseDownEvent, MouseMoveEvent, Window};
use std::rc::Rc;

/// What the pointer needs: the transform as this frame drew it, the view
/// and the canvas.
struct BTransformDrive {
    modal: BTransformModal,
    hit: BHit,
    handler: Rc<dyn Fn(&BAction, &mut Window, &mut App)>,
}

impl BTransformDrive {
    fn moved(&self, event: &MouseMoveEvent, window: &mut Window, app: &mut App) {
        let Some(bounds) = self.hit.bounds.get() else { return };
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        let at = (f32::from(event.position.x - bounds.origin.x), f32::from(event.position.y - bounds.origin.y));
        let view = BPickView { aspect: w / h.max(1.0), ..self.hit.view };
        for action in self.modal.moved(&view, at, (w, h)) {
            (self.handler)(&action, window, app);
        }
    }

    fn pressed(&self, event: &MouseDownEvent, window: &mut Window, app: &mut App) {
        let answer = match event.button {
            MouseButton::Left => "confirm",
            MouseButton::Right => "cancel",
            _ => return,
        };
        (self.handler)(&BAction::Set(OpKey::Transform.path(), answer.to_string()), window, app);
    }
}

/// The surface, while a transform is under way.
pub(super) fn surface(ctx: &BRenderCtx, hit: BHit, modal: BTransformModal) -> gpui::Stateful<Div> {
    let drive = Rc::new(BTransformDrive { modal, hit, handler: ctx.handler.clone() });
    let (on_move, on_down) = (Rc::clone(&drive), drive);
    super::hit::layer("view3d-transform", gpui::CursorStyle::Crosshair, ctx)
        .on_mouse_move(move |event: &MouseMoveEvent, window, app| on_move.moved(event, window, app))
        .on_any_mouse_down(move |event: &MouseDownEvent, window, app| on_down.pressed(event, window, app))
}
