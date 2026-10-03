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
///
/// It takes the context only to describe its box in `BFlex` like everything
/// else: an overlay filling its parent is `BPlace::Overlay`, not two raw
/// calls repeated wherever a tool needs one.
pub(super) fn layer(
    id: &'static str,
    cursor: gpui::CursorStyle,
    ctx: &super::super::super::ctx::BRenderCtx,
) -> gpui::Stateful<gpui::Div> {
    use crate::v2::view::draw::flex::{BFlex, BPlace};
    use gpui::{InteractiveElement as _, Styled as _};
    BFlex::cell()
        .place(BPlace::Overlay)
        .div(ctx)
        .id(gpui::SharedString::from(id))
        .debug_selector(move || id.to_string())
        .cursor(cursor)
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
    /// Shift + right-click: the 3D cursor onto the surface under the
    /// pointer, or over empty space onto the plane through the cursor that
    /// faces the view — turned to the view (`view3d.cursor3d`, `use_depth`,
    /// `orientation='VIEW'`; `scene::cursor`). With Ctrl too it is the
    /// lasso's deselect (`lasso.rs`), as Blender's keymap has it.
    pub fn on_cursor(self, ctx: &BRenderCtx) -> impl Fn(&gpui::MouseDownEvent, &mut Window, &mut App) + 'static {
        let (handler, now) = (ctx.handler.clone(), crate::v2::scene::cursor::BCursor::now(ctx.state));
        move |event: &gpui::MouseDownEvent, window: &mut Window, app: &mut App| {
            let Some(bounds) = self.bounds.get().filter(|_| event.modifiers.shift && !event.modifiers.control) else { return };
            let (x, y) = (f32::from(event.position.x - bounds.origin.x), f32::from(event.position.y - bounds.origin.y));
            let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            let (su, sv) = BPickView::canvas_point(x, y, w, h);
            let view = BPickView { aspect: w / h.max(1.0), ..self.view };
            let Some(placed) = now.under(&self.scene, &view, (su, sv)) else { return };
            (Rc::clone(&handler))(&BAction::Set(crate::v2::state::SceneKey::Cursor.path(), placed.encode()), window, app);
        }
    }

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
