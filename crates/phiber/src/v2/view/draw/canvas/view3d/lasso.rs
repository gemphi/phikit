//! The lasso gesture on the orbit layer (`scene::lasso`): Ctrl with the
//! right button starts an outline, the drag extends it
//! (`ctx.view3d.lasso.path`), and the release selects what is inside —
//! added, or with Shift taken away (`BBoxMode::fire`) — and clears it.
//! [`paint`] draws the outline as `wm_gesture_draw_lasso` does: dashed
//! over solid, closed back to where it began.

use super::hit::BHit;
use crate::v2::scene::boxsel::BBoxMode;
use crate::v2::scene::lasso::BLasso;
use crate::v2::scene::pick::BPickView;
use crate::v2::scene::select::BSelection;
use crate::v2::state::{BKey as _, BState, OpKey, View3dKey, val::V_EMPTY};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{App, Bounds, Div, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PathBuilder, Pixels, Point, Stateful, Window, point, px};
use std::rc::Rc;

type BHandler = Rc<dyn Fn(&BAction, &mut Window, &mut App)>;

/// What the gesture reads, as this frame drew it.
struct BLassoGesture {
    hit: BHit,
    lasso: Option<BLasso>,
    selection: BSelection,
    handler: BHandler,
}

impl BLassoGesture {
    fn local(&self, at: Point<Pixels>) -> Option<(f32, f32)> {
        let bounds = self.hit.bounds.get()?;
        Some((f32::from(at.x - bounds.origin.x), f32::from(at.y - bounds.origin.y)))
    }

    fn set(&self, path: String, window: &mut Window, app: &mut App) {
        (self.handler)(&BAction::Set(View3dKey::LassoPath.path(), path), window, app);
    }

    fn start(&self, e: &MouseDownEvent, window: &mut Window, app: &mut App) {
        let Some((x, y)) = self.local(e.position) else { return };
        if e.modifiers.control || e.modifiers.platform {
            self.set(BLasso { points: vec![(x, y)] }.encode(), window, app);
        }
    }

    fn extend(&self, e: &MouseMoveEvent, window: &mut Window, app: &mut App) {
        let (Some(lasso), Some((x, y))) = (&self.lasso, self.local(e.position)) else { return };
        if e.pressed_button == Some(MouseButton::Right)
            && let Some(longer) = lasso.extended(x, y)
        {
            self.set(longer.encode(), window, app);
        }
    }

    fn finish(&self, e: &MouseUpEvent, window: &mut Window, app: &mut App) {
        let (Some(lasso), Some(bounds)) = (&self.lasso, self.hit.bounds.get()) else { return };
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        let view = BPickView { aspect: w / h.max(1.0), ..self.hit.view };
        let mut names = lasso.names(&self.hit.scene, &view, w, h, |n| !self.hit.unselectable.contains(n));
        names.extend(crate::v2::scene::pick::extra::BPickExtra::origins_where(&self.hit.extras, &view, (w, h), |x, y| lasso.contains(x, y)));
        self.set(V_EMPTY.to_string(), window, app);
        let mode = if e.modifiers.shift { BBoxMode::Sub } else { BBoxMode::Add };
        if let Some(op) = mode.fire(names, |n| self.selection.contains(n)) {
            (self.handler)(&BAction::Set(OpKey::Select.path(), op), window, app);
        }
    }
}

/// The layer, listening for the lasso too.
pub(super) fn wire(layer: Stateful<Div>, s: &BState, handler: BHandler, hit: BHit) -> Stateful<Div> {
    let gesture = Rc::new(BLassoGesture { hit, lasso: BLasso::now(s), selection: BSelection::now(s), handler });
    let (down, moved, up) = (Rc::clone(&gesture), Rc::clone(&gesture), gesture);
    layer
        .on_mouse_down(MouseButton::Right, move |e, window, app| down.start(e, window, app))
        .on_mouse_move(move |e, window, app| moved.extend(e, window, app))
        .on_mouse_up(MouseButton::Right, move |e, window, app| up.finish(e, window, app))
}

/// The outline, closed, in the box's two inks.
pub(super) fn paint(bounds: Bounds<Pixels>, lasso: &BLasso, back: Hsla, ink: Hsla, window: &mut Window) {
    stroke(bounds, &lasso.points, true, back, ink, window);
}

/// A gesture's line through canvas `points` — `closed` back to the first —
/// solid in `back` under a dashed `ink`: the lasso, the transform's guide.
pub(super) fn stroke(bounds: Bounds<Pixels>, points: &[(f32, f32)], closed: bool, back: Hsla, ink: Hsla, window: &mut Window) {
    let at = |(x, y): (f32, f32)| point(bounds.origin.x + px(x), bounds.origin.y + px(y));
    let Some(first) = points.first().copied() else { return };
    for (dash, color) in [(None, back), (Some(super::gesture::DASH), ink)] {
        let mut builder = PathBuilder::stroke(px(super::gesture::LINE));
        if let Some(dash) = dash {
            builder = builder.dash_array(&[px(dash), px(dash)]);
        }
        builder.move_to(at(first));
        for p in points.iter().skip(1) {
            builder.line_to(at(*p));
        }
        if closed {
            builder.line_to(at(first));
        }
        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
    }
}
