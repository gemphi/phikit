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

/// phiber's canvas has no zoom, so the hit-test's scale is 1. Named rather than
/// spelled inline so the day the canvas zooms there is one place to change.
const SCALE: f32 = 1.0;

/// One render's worth of link gesturing.
pub struct BWire {
    pub graph: BlendNodeGraph,
    /// The links as drawn — the file's, with this host's edits applied.
    pub wired: Vec<BlendNodeLink>,
    /// Each node's drawn position, as fractions of the canvas.
    pub placed: Vec<(String, (f32, f32))>,
    pub edits: link::BLinkEdits,
    pub drag: Option<BLinking>,
    pub handler: BActionHandler,
    pub host: BHostType,
    pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    pub header: f32,
    pub unit: f32,
    pub card_width: f32,
}

impl BWire {
    /// The cursor in canvas pixels, and the canvas's size — `None` before the
    /// first paint, or outside it.
    fn locate(&self, at: Point<Pixels>) -> Option<((f32, f32), (f32, f32))> {
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
    fn cards(&self, size: (f32, f32)) -> Vec<super::paint::BNodeCard> {
        super::plan::plan(
            &self.graph,
            &self.wired,
            &self.placed,
            size.0,
            size.1,
            self.header,
            self.unit,
            self.card_width,
        )
        .0
    }

    fn write(&self, key: NodeKey, value: String, window: &mut Window, app: &mut App) {
        // `Adjust`, not `Set`: a gesture writes continuously, and a committed
        // `Set` would take every open overlay down on the first pixel.
        (self.handler)(&BAction::Adjust(key.path(), value), window, app);
    }

    /// A press: if it lands on a socket, start carrying a link.
    ///
    /// Blender searches for an OUTPUT first and an input only if that misses
    /// (`node_link_init` calls `node_find_indicated_socket` with `SOCK_OUT`
    /// and then `SOCK_IN`), so a cursor near both grabs the output. That order
    /// is the whole reason the two searches are separate calls here too.
    pub fn down(&self, event: &gpui::MouseDownEvent, window: &mut Window, app: &mut App) {
        let Some((cursor, size)) = self.locate(event.position) else { return };
        let cards = self.cards(size);
        let found = pick::socket_at(&cards, cursor, Some(BSocketSide::Output), SCALE)
            .map(|hit| (hit, true))
            .or_else(|| {
                pick::socket_at(&cards, cursor, Some(BSocketSide::Input), SCALE)
                    .map(|hit| (hit, false))
            });
        // No socket: the press belongs to whatever is under it — a card's own
        // click handler selects it, and the canvas keeps out of the way.
        let Some(((index, row), output)) = found else { return };
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
        let Some(drag) = &self.drag else { return };
        let target = self.locate(at).and_then(|(cursor, size)| {
            let cards = self.cards(size);
            let (index, row) = pick::socket_at(&cards, cursor, Some(drag.wants()), SCALE)?;
            Some(BLinkEnd::new(&cards.get(index)?.name, &row.identifier))
        });
        if let Some(next) =
            link::commit(&self.graph, &self.wired, drag, target.as_ref(), &self.edits)
        {
            self.write(NodeKey::Wired(self.host), link::encode(&next), window, app);
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
        .when(carrying, |element| {
            element.on_mouse_move(move |event: &gpui::MouseMoveEvent, window, app| {
                moved.moved(event.position, window, app)
            })
        })
}

#[cfg(test)]
mod tests;
