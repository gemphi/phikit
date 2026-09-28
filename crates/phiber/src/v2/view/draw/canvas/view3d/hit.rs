//! The click that selects: a left click on the canvas — one that did not
//! turn into an orbit drag — picks the object under the pointer
//! (`scene::pick`) and makes it the active object, which the Outliner, the
//! Properties editor and the overlay all read (`ctx.object`).
//!
//! Blender ref: `view3d_select.cc::view3d_select_exec` with the default
//! left-click select keymap: `deselect_all` is on, so a click on nothing
//! deselects everything — the active object stays active, unselected, as
//! Blender keeps it.

use super::super::super::ctx::BRenderCtx;
use crate::v2::scene::BObject3D;
use crate::v2::scene::pick::{BPickView, pick_where};
use crate::v2::state::{BKey as _, CtxKey, OpKey};
use crate::v2::view::prim::BAction;
use gpui::{App, Bounds, ClickEvent, Pixels, Window};
use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

/// The gesture layer over the canvas every viewport tool draws on — the
/// orbit, the box, the circle brush: the whole canvas, `cursor` over it.
pub(super) fn layer(id: &'static str, cursor: gpui::CursorStyle) -> gpui::Stateful<gpui::Div> {
    use gpui::{InteractiveElement as _, Styled as _};
    gpui::div().id(gpui::SharedString::from(id)).debug_selector(move || id.to_string()).absolute().inset_0().cursor(cursor)
}

/// What a click needs to find its object: the visible scene, the objects in
/// it the arrow toggle keeps from being picked, the view it is drawn in and
/// the canvas's bounds as last painted.
#[derive(Clone)]
pub(super) struct BHit {
    pub scene: Rc<Vec<BObject3D>>,
    /// The cameras, lights and empties drawn and selectable (`pick::extra`).
    pub extras: Rc<Vec<crate::v2::scene::pick::extra::BPickExtra>>,
    pub unselectable: Rc<HashSet<String>>,
    pub view: BPickView,
    pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl BHit {
    /// The click handler: pick under the pointer, set the active object.
    pub fn on_click(self, ctx: &BRenderCtx) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        let handler = ctx.handler.clone();
        move |event: &ClickEvent, window: &mut Window, app: &mut App| {
            let Some(bounds) = self.bounds.get() else { return };
            let position = event.position();
            let (x, y) = (f32::from(position.x - bounds.origin.x), f32::from(position.y - bounds.origin.y));
            let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            let (su, sv) = BPickView::canvas_point(x, y, w, h);
            let view = BPickView { aspect: w / h.max(1.0), ..self.view };
            // A camera, light or empty drawn under the pointer comes first —
            // its lines are overlays over the meshes; else the mesh the ray hits.
            use crate::v2::scene::pick::extra::{BPickExtra, REACH};
            let extra = BPickExtra::nearest(&self.extras, &view, (x, y), (w, h), REACH).map(str::to_string);
            let picked = extra.or_else(|| pick_where(&self.scene, view.ray(su, sv), |name| !self.unselectable.contains(name)).map(|hit| hit.name));
            match picked {
                Some(name) => {
                    // Shift extends the selection (`extend = True`); the click
                    // then makes the picked object the active one.
                    if event.modifiers().shift {
                        (Rc::clone(&handler))(&BAction::Set(OpKey::Select.path(), format!("extend:{name}")), window, app);
                    }
                    (Rc::clone(&handler))(&BAction::Set(CtxKey::Object.path(), name), window, app);
                }
                // Empty canvas: deselect all, unless Shift is held.
                None if !event.modifiers().shift => {
                    (Rc::clone(&handler))(&BAction::Set(OpKey::Select.path(), "none".to_string()), window, app);
                }
                None => {}
            }
        }
    }
}
