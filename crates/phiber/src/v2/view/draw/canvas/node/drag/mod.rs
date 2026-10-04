//! Dragging a node across the canvas.
//!
//! The editor drew the material's real tree but drew it read-only. Blender's node
//! editor is mostly moving nodes around — G is `NODE_OT_translate_attach`, the
//! first thing anyone does to a graph — so a graph you cannot rearrange is a
//! picture of one.
//!
//! Three facts from Blender 5.2.0, `editors/transform/transform_convert_node.cc`:
//!
//! * **A node's origin is its top-left corner**, not its centre:
//!   `create_transform_data_for_node` — "Use top-left corner as the transform
//!   origin for nodes." `td2d.loc` is `node.location` unchanged.
//! * **A move is a translation by delta from where the node started**, not a jump
//!   to the pointer: `td.iloc` is the initial location and `flushTransNodes` adds
//!   the accumulated offset to it. So grabbing a card by its corner does not snap
//!   that corner under the cursor.
//! * **Snapping rounds the start and the offset separately**: `node_snap_grid_apply`,
//!   "so that multiple unsnapped nodes snap to the grid in sync while moving".
//!   phiber has no node snap setting, so nothing here rounds.
//!
//! Two divergences used to be recorded here, both of them consequences of the
//! canvas FITTING the graph rather than panning and zooming it. The view
//! (`view/`) is wired now, so both are gone and it is worth saying how, because
//! they read as facts about dragging and were facts about the framing:
//!
//! 1. **The displacement was in canvas fractions.** It is in Blender units now,
//!    the same units `bNode.location` is in. The fraction existed because a
//!    displacement in units would have fed back into the fit, reframing the graph
//!    under the pointer mid-drag; there is no fit to feed any more.
//! 2. **A node could not be dragged off the canvas.** It can now, because you
//!    can pan back to it — which is exactly Blender's reason for allowing it.
//!    [`placed`] no longer clamps, and `LIMIT` is gone with the band it named.
//!
//! The drag writes `Adjust`, not `Set`: a move is not a decision, and a `Set` per
//! mouse move would take every open overlay down on the first pixel. Nothing
//! commits on release — the value written during the drag is already the value,
//! so a confirming `Set` would only close menus unrelated to the canvas.

use super::super::super::ctx::BRenderCtx;
use super::view::BNodeView;
use crate::v2::host::BHostType;
use crate::v2::state::{BKey as _, BState, NodeKey};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{App, Context, Div, DragMoveEvent, Pixels, Point, Render, Stateful, Window};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// The drag in flight — one node: phiber's selection holds a single name (`NodeKey::Selected`), and
/// Blender's `get_transformed_nodes` moves every node with `NODE_SELECT` set,
/// which with a selection of one is the same node. When box select arrives this
/// grows a list; it is not pretending to have one now.
#[derive(Clone, Debug, PartialEq)]
pub struct BNodeDrag {
    /// `bNode.name`, interned — the node being moved.
    pub name: &'static str,
    /// Its displacement when the drag began, so each write is `from + delta`
    /// rather than an accumulation that drifts over a long drag.
    pub from: (f32, f32),
    /// The pointer when the first move arrived, as a canvas fraction.
    ///
    /// Still a fraction, and deliberately: the delta is converted to view units
    /// against the view this render was built with, so a pan or zoom arriving
    /// mid-drag cannot retroactively rescale the part of the drag already done.
    ///
    /// `None` until then: `on_drag`'s start callback is handed the pointer but
    /// not the canvas's bounds, and a fraction needs both, so there is nothing
    /// honest to compute at the press — the gap `track::draggable` also records.
    /// Taking the origin from the first move is what makes this a delta drag,
    /// which is Blender's behaviour anyway.
    pub grabbed: Option<(f32, f32)>,
}

impl BNodeDrag {
    pub fn new(name: &'static str, from: (f32, f32)) -> Self {
        Self { name, from, grabbed: None }
    }
}

/// `"dx, dy"` — the spelling `encode_hsla` uses for a colour's channels, so the
/// state log holds one format for a tuple and not two.
pub fn encode(offset: (f32, f32)) -> String {
    format!("{}, {}", offset.0, offset.1)
}

/// `"dx, dy"` back to a pair. Anything else reads as no displacement: a node
/// whose key has been edited into nonsense should appear where the file put it
/// rather than vanish.
pub fn decode(raw: &str) -> Option<(f32, f32)> {
    let (x, y) = raw.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

/// This node's stored displacement, or none at all.
pub fn offset_of(state: &BState, host: BHostType, name: &str) -> (f32, f32) {
    let key = NodeKey::At(host, crate::v2::state::intern(name));
    state.get(&key.path()).and_then(decode).unwrap_or((0.0, 0.0))
}

/// Where a node is, in view units: the file's place plus its displacement.
///
/// No clamp. There used to be one, because a node dragged past the edge of a
/// canvas that could not pan was gone for good. The view pans, so a node off
/// screen is a node you scroll to — which is what Blender does.
///
/// A non-finite displacement still reads as none: a key edited into nonsense
/// should leave the node where the file put it rather than send it to infinity.
pub fn placed(base: (f32, f32), offset: (f32, f32)) -> (f32, f32) {
    let add = |a: f32, b: f32| if (a + b).is_finite() { a + b } else { a };
    (add(base.0, offset.0), add(base.1, offset.1))
}

/// The displacement to WRITE for a pointer that has moved `delta` view units
/// since it grabbed, given what the node had already moved by.
///
/// `flushTransNodes` adds the accumulated offset to `td.iloc`, so this is a sum
/// and not an accumulation: each write is `from + delta` measured from the grab,
/// which is what keeps a long drag from drifting.
pub fn moved(from: (f32, f32), delta: (f32, f32)) -> (f32, f32) {
    (from.0 + delta.0, from.1 + delta.1)
}

/// A pointer delta in canvas fractions, as view units.
///
/// The X is the fraction times the view's width — the cancellation
/// `BNodeView::pan_fraction` documents. The Y is NEGATED, because the canvas
/// grows downward and `bNode.location` grows up: dragging a card down the screen
/// must lower the node's `y`, not raise it. This is the only place in the drag
/// that knows which way Y runs, and the one place it can be got wrong.
pub fn in_view_units(delta: (f32, f32), view: &BNodeView) -> (f32, f32) {
    (delta.0 * view.width(), -delta.1 * view.height())
}

/// Nothing leaves the card — the node moves, it is not dropped onto anything —
/// so the gesture's drag image is empty, as `BTrackGhost` is.
struct BNodeGhost;

impl Render for BNodeGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

/// Make `card` the grab handle for `drag`.
pub fn grab(card: Stateful<Div>, drag: BNodeDrag) -> Stateful<Div> {
    card.cursor(gpui::CursorStyle::OpenHand).on_drag(
        Arc::new(Mutex::new(drag)),
        move |state, _: Point<Pixels>, _window, cx| {
            // A payload can outlive one render of the card, so the grab origin
            // is cleared here rather than trusted to be fresh.
            if let Ok(mut state) = state.lock() {
                state.grabbed = None;
            }
            cx.new(|_| BNodeGhost)
        },
    )
}

/// Let `canvas` turn the pointer into a displacement for whichever card is being
/// dragged over it.
///
/// The handler belongs to the CANVAS, not the card: `DragMoveEvent::bounds` is
/// the listening element's own bounds, and a fraction of the canvas is the only
/// thing a fitted position can be measured against. Hung on the card the bounds
/// would be the card's — and the card moves as the drag proceeds, so every move
/// would be measured against a different rectangle.
///
/// `moving` is what the drag carries, captured for this render; `view` is the
/// framing those fractions are measured against, so the pointer's travel across
/// the canvas becomes travel in node space.
pub fn follow(canvas: Div, moving: Vec<BNodeMove>, view: BNodeView, ctx: &BRenderCtx) -> Div {
    let handler = ctx.handler.clone();
    let host = ctx.host;
    let moving = Rc::new(moving);
    canvas.on_drag_move::<Arc<Mutex<BNodeDrag>>>(
        move |event: &DragMoveEvent<Arc<Mutex<BNodeDrag>>>, window, app: &mut App| {
            let bounds = event.bounds;
            let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            if w <= 0.0 || h <= 0.0 {
                return;
            }
            let at = event.event.position;
            let now = (
                (f32::from(at.x) - f32::from(bounds.origin.x)) / w,
                (f32::from(at.y) - f32::from(bounds.origin.y)) / h,
            );
            let state = Arc::clone(event.drag(app));
            let Ok(mut drag) = state.lock() else { return };
            let grabbed = *drag.grabbed.get_or_insert(now);
            let delta = in_view_units((now.0 - grabbed.0, now.1 - grabbed.1), &view);
            drop(drag);
            // Every node in `moving` gets the same delta — `get_transformed_nodes`
            // keeps *"only nodes that are selected"* and `flushTransNodes` adds
            // one offset to each of their own `td.iloc`, so a group moves
            // rigidly rather than converging on the pointer.
            for node in moving.iter() {
                let offset = moved(node.from, delta);
                let key = NodeKey::At(host, node.name);
                (Rc::clone(&handler))(&BAction::Adjust(key.path(), encode(offset)), window, app);
            }
        },
    )
}

/// One node a drag will move: its name, where the fit puts it, and the
/// displacement it already carries.
///
/// Gathered at render time because the move handler cannot read the store — and
/// because `get_transformed_nodes` is evaluated once when the transform begins
/// in Blender too, not re-derived on every mouse move.
#[derive(Clone, Debug, PartialEq)]
pub struct BNodeMove {
    pub name: &'static str,
    pub from: (f32, f32),
}

#[cfg(test)]
mod tests;
