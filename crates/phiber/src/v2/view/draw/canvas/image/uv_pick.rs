//! Picking and moving UVs over the editor's square — `uv_select_pick`
//! (`uvedit_select.cc`) and a transform over `TC_UV`.
//!
//! A left click takes the nearest corner within a radius, as Blender does
//! (`ED_uvedit_nearest_uv` over `SNAP_DIST`), replacing the selection —
//! with Shift it is added, or dropped if it was already in it. A left drag
//! that began on a selected corner moves the whole selection with the
//! pointer; one that began on empty square does nothing, so a stray drag
//! cannot throw a layout about.
//!
//! The move is written as an **offset per corner** and folded into whatever
//! is already there (`scene::uv_edit`), so the loaded model stays the
//! file's and a second drag continues the first.
//!
//! **Where the drag's anchor lives matters.** It is a state key
//! (`UvKey::Drag`), not a cell on the gesture: the press writes the
//! selection, that write re-renders, the re-render builds a new gesture —
//! and any cell the old one's handlers were holding goes with it. A cell
//! works for a gesture that writes nothing until it ends (the render
//! region's box, `border.rs`); it silently does not for one that writes on
//! the press.

use crate::v2::part::image::uv::BUvShown;
use crate::v2::scene::uv_edit::{BUvCorner, BUvEdit};
use crate::v2::state::{BKey as _, UvKey};
use crate::v2::view::draw::ctx::{BActionHandler, BRenderCtx};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{App, Bounds, Pixels, Point, Window, px};
use std::cell::Cell;
use std::rc::Rc;

/// How near the pointer has to be, in pixels, for a corner to be picked —
/// Blender's `ED_uvedit_nearest_uv` searches the whole region and then
/// applies the view's own threshold; a fixed radius is that threshold at
/// the editor's own scale.
const SNAP: f32 = 12.0;
/// `UvKey::Drag`'s two numbers.
const DRAG_SEP: char = ':';

fn read_drag(raw: Option<&str>) -> Option<[f32; 2]> {
    let (u, v) = raw?.split_once(DRAG_SEP)?;
    Some([u.parse().ok()?, v.parse().ok()?])
}

/// The gesture over one UV editor's square.
#[derive(Clone)]
pub struct BUvGesture {
    /// Edit Mode on something with UVs; otherwise the square is read-only.
    pub edits: bool,
    pub shown: Rc<BUvShown>,
    pub edit: BUvEdit,
    /// The 0..1 square, set by the painter each frame.
    pub square: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// Where the pointer was when the drag last moved, as a UV — the state
    /// key's value, read fresh each frame.
    pub drag: Option<[f32; 2]>,
    pub handler: BActionHandler,
}

impl BUvGesture {
    pub fn of(ctx: &BRenderCtx, shown: Rc<BUvShown>, square: Rc<Cell<Option<Bounds<Pixels>>>>) -> Self {
        Self {
            edits: shown.edits(),
            shown,
            edit: BUvEdit::now(ctx.state),
            square,
            drag: read_drag(ctx.state.get(UvKey::Drag)),
            handler: ctx.handler.clone(),
        }
    }

    /// A point on the square as a UV, `u` right and `v` up.
    fn uv_at(square: Bounds<Pixels>, at: Point<Pixels>) -> [f32; 2] {
        let (w, h) = (f32::from(square.size.width).max(1.0), f32::from(square.size.height).max(1.0));
        [f32::from(at.x - square.origin.x) / w, 1.0 - f32::from(at.y - square.origin.y) / h]
    }

    /// A UV as a point on the square.
    fn point_at(square: Bounds<Pixels>, [u, v]: [f32; 2]) -> Point<Pixels> {
        let (w, h) = (f32::from(square.size.width), f32::from(square.size.height));
        gpui::point(square.origin.x + px(u * w), square.origin.y + px((1.0 - v) * h))
    }

    /// The corner nearest `at` within [`SNAP`], its UVs read as they now
    /// stand so a moved corner is picked where it is drawn, not where the
    /// file put it.
    pub fn nearest(&self, square: Bounds<Pixels>, at: Point<Pixels>) -> Option<BUvCorner> {
        let mut best: Option<(f32, BUvCorner)> = None;
        for (m, map) in self.shown.maps.iter().enumerate() {
            for (c, uv) in map.uvs.iter().enumerate() {
                let p = Self::point_at(square, self.edit.uv_of((m, c), *uv));
                let (dx, dy) = (f32::from(p.x - at.x), f32::from(p.y - at.y));
                let d2 = dx * dx + dy * dy;
                if d2 <= SNAP * SNAP && best.is_none_or(|(b, _)| d2 < b) {
                    best = Some((d2, (m, c)));
                }
            }
        }
        best.map(|(_, corner)| corner)
    }

    /// The element with the gesture's handlers on it.
    pub fn attach(self, element: gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> {
        if !self.edits {
            return element;
        }
        let (down, moved, up) = (self.clone(), self.clone(), self);
        element
            .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| down.press(event, window, app))
            .on_mouse_move(move |event, window, app| moved.mouse_move(event, window, app))
            .on_mouse_up(gpui::MouseButton::Left, move |_, window, app| up.release(window, app))
    }

    /// A press: the corner under the pointer is picked, and the drag is
    /// armed only when the press landed on one of the selected corners —
    /// which is what makes a drag over empty square a no-op.
    pub fn press(&self, event: &gpui::MouseDownEvent, window: &mut Window, app: &mut App) {
        let Some(square) = self.square.get() else { return };
        let Some(corner) = self.nearest(square, event.position) else { return };
        let list = self.edit.picked(corner, event.modifiers.shift);
        let [u, v] = Self::uv_at(square, event.position);
        (self.handler)(&BAction::Set(UvKey::Selected.path(), BUvEdit::write_selected(&list)), window, app);
        (self.handler)(&BAction::Set(UvKey::Drag.path(), format!("{u}{DRAG_SEP}{v}")), window, app);
    }

    /// The drag ends; the selection and the offsets stay as they are.
    pub fn release(&self, window: &mut Window, app: &mut App) {
        if self.drag.is_some() {
            (self.handler)(&BAction::Set(UvKey::Drag.path(), String::new()), window, app);
        }
    }

    /// A move with the button down: the selection follows the pointer, the
    /// step written as the UVs it comes to on the square.
    pub fn mouse_move(&self, event: &gpui::MouseMoveEvent, window: &mut Window, app: &mut App) {
        let (Some(was), Some(square)) = (self.drag, self.square.get()) else { return };
        if event.pressed_button != Some(gpui::MouseButton::Left) {
            self.release(window, app);
            return;
        }
        let now = Self::uv_at(square, event.position);
        let by = [now[0] - was[0], now[1] - was[1]];
        if by[0] == 0.0 && by[1] == 0.0 {
            return;
        }
        let moved = self.edit.nudged(by);
        (self.handler)(&BAction::Set(UvKey::Moved.path(), BUvEdit::write_moved(&moved)), window, app);
        (self.handler)(&BAction::Set(UvKey::Drag.path(), format!("{}{DRAG_SEP}{}", now[0], now[1])), window, app);
    }
}
