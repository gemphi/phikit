//! What the pointer does to a link: press on a socket, carry, release.
//!
//! Blender's `NODE_OT_link` is a modal operator and this is the same three
//! steps — `node_link_init` on the press (`link::grab`),
//! `node_link_find_socket` on the move (the cursor is written and the preview
//! redrawn), `add_dragged_links_to_tree` on the release (`link::commit`).
//!
//! It is driven from the CANVAS's mouse events rather than a GPUI drag payload
//! because a socket is painted, not an element: there is no div under one to
//! grab. That turns out to be the closer parity as well as the only option.
//!
//! The canvas's bounds arrive through a `Cell` that the paint pass fills,
//! because a mouse event carries window coordinates and the canvas's own
//! rectangle is not known until it paints — the same arrangement the animation
//! window uses (`canvas/time/gesture.rs`).

use super::link::{self, BLinkEnd, BLinking};
use super::select;
use super::pick;
use super::socket::BSocketSide;
use super::super::super::ctx::{BActionHandler, BRenderCtx};
use crate::phiphx::blend::shade::graph::{BlendNodeGraph, BlendNodeLink};
use crate::v2::host::BHostType;
use crate::v2::state::{BKey as _, NodeKey};
use crate::v2::view::prim::BAction;
use gpui::{App, Bounds, Pixels, Point, Window};
use std::cell::Cell;
use std::rc::Rc;

/// One render's worth of link gesturing.
pub struct BWire {
    pub graph: BlendNodeGraph,
    /// The links as drawn — the file's, with this host's edits applied.
    pub wired: Vec<BlendNodeLink>,
    /// Each node's drawn position, as fractions of the canvas.
    pub placed: Vec<(String, (f32, f32))>,
    pub edits: link::BLinkEdits,
    pub drag: Option<BLinking>,
    pub selection: select::BNodeSelection,
    /// Where a box select began, in canvas fractions — `None` when no box is
    /// being swept.
    pub box_from: Option<(f32, f32)>,
    pub handler: BActionHandler,
    pub host: BHostType,
    pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// A card's size in view units (`plan::BCardSize`).
    pub size: super::plan::BCardSize,
    /// What this editor is looking at — the framing every fraction here is
    /// measured against, and what a pan or a zoom rewrites.
    pub view: super::view::BNodeView,
}

impl BWire {
    /// The cursor in canvas pixels, and the canvas's size — `None` before the
    /// first paint, or outside it.
    pub(super) fn locate(&self, at: Point<Pixels>) -> Option<((f32, f32), (f32, f32))> {
        let bounds = self.bounds.get()?;
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let cursor =
            (f32::from(at.x) - f32::from(bounds.origin.x), f32::from(at.y) - f32::from(bounds.origin.y));
        Some((cursor, (w, h)))
    }

    /// The cards as the painter has them, so the hit-test and the picture agree.
    pub(super) fn cards(&self, region: (f32, f32)) -> Vec<super::paint::BNodeCard> {
        super::plan::plan(
            &self.graph,
            &self.wired,
            &self.placed,
            region.0,
            region.1,
            self.size,
            &self.view,
        )
        .0
    }

    /// The hit-test's `view2d_scale` — a real number now.
    ///
    /// `node_find_indicated_socket` divides twenty screen pixels by this to get
    /// the grab slack in view units, so that the area under the pointer stays
    /// the same size however far you have zoomed. It was pinned at 1 while the
    /// canvas could not zoom; the whole point of that note was that one line
    /// would change, and this is the line.
    pub(super) fn scale(&self, region: (f32, f32)) -> f32 {
        self.view.zoom(region.0)
    }

    pub(super) fn write(&self, key: NodeKey, value: String, window: &mut Window, app: &mut App) {
        // `Adjust`, not `Set`: a gesture writes continuously, and a committed
        // `Set` would take every open overlay down on the first pixel.
        (self.handler)(&BAction::Adjust(key.path(), value), window, app);
    }

    /// A press: if it lands on a socket, start carrying a link.
    ///
    /// Blender searches for an OUTPUT first and an input only if that misses
    /// (`node_link_init`), so a cursor near both grabs the output — which is why
    /// the two searches are separate calls here too.
    pub fn down(&self, event: &gpui::MouseDownEvent, window: &mut Window, app: &mut App) {
        let Some((cursor, size)) = self.locate(event.position) else { return };
        let cards = self.cards(size);
        let scale = self.scale(size);
        let found = pick::socket_at(&cards, cursor, Some(BSocketSide::Output), scale)
            .map(|hit| (hit, true))
            .or_else(|| {
                pick::socket_at(&cards, cursor, Some(BSocketSide::Input), scale)
                    .map(|hit| (hit, false))
            });
        // No socket: the press is a selection, or the start of a box. One drag
        // means three things in Blender (`node_box_select_invoke` passes through
        // over a node or socket): a link off a socket, a move off a card, a box
        // off the canvas — the same three-way split, in the same order.
        let Some(((index, row), output)) = found else {
            return self.press_on_canvas(cursor, size, event, window, app);
        };
        let Some(node) = cards
            .get(index)
            .and_then(|card| self.graph.nodes.iter().find(|n| n.name == card.name))
        else {
            return;
        };
        let sockets = if output { &node.outputs } else { &node.inputs };
        let Some(socket) = sockets.iter().find(|s| s.identifier == row.identifier) else { return };
        let (anchor, moving) = match link::grab(&self.wired, node, socket, output) {
            link::BLinkGrab::Fresh(end) => (end, None),
            link::BLinkGrab::Moved { anchor, was } => (anchor, Some(was)),
        };
        // A picked-up link is dragged by the end OPPOSITE the one it was
        // grabbed from, so the anchor's side is the far end's side.
        let from_output = output || moving.is_some();
        let at = (cursor.0 / size.0, cursor.1 / size.1);
        let drag = BLinking { anchor, from_output, at, moving };
        self.write(NodeKey::Linking(self.host), link::live::encode(&drag), window, app);
    }

    /// Carrying: the cursor moves, so the preview follows it.
    pub fn moved(&self, at: Point<Pixels>, window: &mut Window, app: &mut App) {
        let Some(drag) = &self.drag else { return };
        let Some((cursor, size)) = self.locate(at) else { return };
        let next = BLinking { at: (cursor.0 / size.0, cursor.1 / size.1), ..drag.clone() };
        self.write(NodeKey::Linking(self.host), link::live::encode(&next), window, app);
    }

    /// The release: commit if it landed on a socket, and clear the drag either
    /// way — a carried link never outlives the button.
    pub fn up(&self, at: Point<Pixels>, window: &mut Window, app: &mut App) {
        if self.drag.is_none() {
            return self.release_on_canvas(at, window, app);
        }
        let Some(drag) = &self.drag else { return };
        let target = self.locate(at).and_then(|(cursor, size)| {
            let cards = self.cards(size);
            let (index, row) = pick::socket_at(&cards, cursor, Some(drag.wants()), self.scale(size))?;
            Some(BLinkEnd::new(&cards.get(index)?.name, &row.identifier))
        });
        if let Some(next) =
            link::commit(&self.graph, &self.wired, drag, target.as_ref(), &self.edits)
        {
            self.write(NodeKey::Wired(self.host), link::encode(&next), window, app);
        }
        // A fresh link (not one dragged off to delete) dropped on empty canvas
        // opens the Add search wired to its end (`link_drag_search`).
        if target.is_none() && drag.moving.is_none() {
            self.open_add_search(drag, window, app);
        }
        // `Reset` rather than an empty `Adjust`: an absent key is the honest
        // spelling of "no drag", and it leaves nothing behind in the store.
        (self.handler)(&BAction::Reset(NodeKey::Linking(self.host).path()), window, app);
    }
}

/// Hang the three handlers on the canvas. `wire` is `None` when there is no
/// graph, in which case the canvas has no sockets and nothing to listen for.
pub fn listen(canvas: gpui::Div, wire: Option<Rc<BWire>>, _ctx: &BRenderCtx) -> gpui::Div {
    use gpui::{InteractiveElement, MouseButton};
    use gpui::prelude::FluentBuilder as _;
    let Some(wire) = wire else { return canvas };
    let carrying = wire.drag.is_some();
    let (down, moved, up, out) =
        (Rc::clone(&wire), Rc::clone(&wire), Rc::clone(&wire), Rc::clone(&wire));
    canvas
        .on_mouse_down(MouseButton::Left, move |event, window, app| down.down(event, window, app))
        .on_mouse_up(MouseButton::Left, move |event: &gpui::MouseUpEvent, window, app| {
            up.up(event.position, window, app)
        })
        // Released off the canvas: still a release, and it must clear the drag
        // or the preview would hang there until the next press.
        .on_mouse_up_out(MouseButton::Left, move |event: &gpui::MouseUpEvent, window, app| {
            out.up(event.position, window, app)
        })
        // The `View2D` keymap: the middle button pans, the wheel zooms about the
        // cursor. Both are the canvas's, not a card's — a pan started over a
        // node is still a pan, which is why these sit beside the link handlers
        // rather than on the cards.
        .on_mouse_down(MouseButton::Middle, {
            let wire = Rc::clone(&wire);
            move |event: &gpui::MouseDownEvent, _window, _app| wire.pan_start(event.position)
        })
        .on_mouse_up(MouseButton::Middle, {
            let wire = Rc::clone(&wire);
            move |_event: &gpui::MouseUpEvent, _window, _app| wire.pan_end()
        })
        // Released off the canvas: still a release, or the pan would resume on
        // the next move over it.
        .on_mouse_up_out(MouseButton::Middle, {
            let wire = Rc::clone(&wire);
            move |_event: &gpui::MouseUpEvent, _window, _app| wire.pan_end()
        })
        .on_scroll_wheel({
            let wire = Rc::clone(&wire);
            move |event: &gpui::ScrollWheelEvent, window, app| wire.zoom(event, window, app)
        })
        .when(carrying || BWire::panning(), |element| {
            let wire = Rc::clone(&wire);
            element.on_mouse_move(move |event: &gpui::MouseMoveEvent, window, app| {
                // One move handler for both gestures: a carried link follows the
                // cursor, a pan carries the view. They cannot both be in flight
                // — a pan takes the middle button and a link the left.
                wire.pan_move(event.position, window, app);
                moved.moved(event.position, window, app);
            })
        })
}

pub use build::{BFrame, for_frame};

/// Selecting cards and sweeping a box — the half of the gesture that is not
/// about links.
mod choose;
/// Assembling one frame's gesture state from what the renderer already has.
mod build;
/// Panning and zooming — the `View2D` keymap.
mod frame;

#[cfg(test)]
mod tests;
