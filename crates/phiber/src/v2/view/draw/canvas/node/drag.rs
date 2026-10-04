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
//! Two divergences, both from the canvas FITTING the graph rather than panning
//! and zooming it (the divergence recorded in `mod.rs`):
//!
//! 1. **The displacement is in canvas fractions, not Blender units.** Everything
//!    on a fitted canvas is a fraction, so a window resize rescales a drag along
//!    with the graph it belongs to. A displacement in Blender units would feed
//!    back into `BExtent`, and then dragging a node would reframe the graph under
//!    the pointer and the card would not follow the cursor — which is why the
//!    extent is computed from FILE positions alone.
//! 2. **A node cannot be dragged off the canvas.** Blender lets you, because you
//!    can always pan back; with no pan a node past the edge is gone for good, so
//!    [`placed`] clamps into [`LIMIT`], the band the fit already reserves.
//!
//! The drag writes `Adjust`, not `Set`: a move is not a decision, and a `Set` per
//! mouse move would take every open overlay down on the first pixel. Nothing
//! commits on release — the value written during the drag is already the value,
//! so a confirming `Set` would only close menus unrelated to the canvas.

use super::super::super::ctx::BRenderCtx;
use super::{FIT, INSET};
use crate::v2::host::BHostType;
use crate::v2::state::{BKey as _, BState, NodeKey};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{App, Context, Div, DragMoveEvent, Pixels, Point, Render, Stateful, Window};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// The furthest a card's top-left corner may sit, as a fraction of the canvas.
///
/// The fit places nodes between `INSET` and `INSET + FIT`, leaving the rest as
/// the room a card's own width and height need. A dragged node gets that same
/// band, so one that has moved is no likelier to run off the edge than one that
/// never did.
pub const LIMIT: f32 = INSET + FIT;

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

/// Where a node is drawn: fitted place plus displacement, clamped to [`LIMIT`].
pub fn placed(base: (f32, f32), offset: (f32, f32)) -> (f32, f32) {
    let clamp = |v: f32| if v.is_finite() { v.clamp(0.0, LIMIT) } else { 0.0 };
    (clamp(base.0 + offset.0), clamp(base.1 + offset.1))
}

/// The displacement to WRITE for a pointer that has moved `delta` since it
/// grabbed, given where the node was fitted and what it had already moved by.
/// The clamp applies here and not only in [`placed`], so the STORED value is the
/// one honoured. Clamping at draw time alone would let a drag past the edge bank
/// up displacement the canvas never showed, and the node would then sit still
/// through the first half of the drag back.
pub fn moved(base: (f32, f32), from: (f32, f32), delta: (f32, f32)) -> (f32, f32) {
    let at = placed(base, (from.0 + delta.0, from.1 + delta.1));
    (at.0 - base.0, at.1 - base.1)
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
/// `fitted` is each node's fitted position by name, captured for this render, so
/// the clamp knows what the displacement is relative to.
pub fn follow(canvas: Div, fitted: Vec<(String, (f32, f32))>, ctx: &BRenderCtx) -> Div {
    let handler = ctx.handler.clone();
    let host = ctx.host;
    let fitted = Rc::new(fitted);
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
            let Some((_, base)) = fitted.iter().find(|(name, _)| name == drag.name) else { return };
            let offset = moved(*base, drag.from, (now.0 - grabbed.0, now.1 - grabbed.1));
            let key = NodeKey::At(host, drag.name);
            drop(drag);
            (Rc::clone(&handler))(&BAction::Adjust(key.path(), encode(offset)), window, app);
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_displacement_round_trips() {
        for offset in [(0.0, 0.0), (0.25, -0.125), (-0.5, 0.5)] {
            assert_eq!(decode(&encode(offset)), Some(offset), "{offset:?}");
        }
        assert_eq!(decode("banana"), None, "nonsense is no displacement, not a panic");
        assert_eq!(decode("0.5"), None, "a pair needs both halves");
    }

    #[test]
    fn a_node_moves_by_the_delta_it_was_dragged() {
        // `td.iloc` + offset: the card keeps the relationship to the pointer it
        // had at the press.
        let base = (0.2, 0.2);
        let offset = moved(base, (0.0, 0.0), (0.1, -0.05));
        assert!((offset.0 - 0.1).abs() < 1e-6 && (offset.1 + 0.05).abs() < 1e-6, "{offset:?}");
        // A second drag starts from where the first left it.
        assert!((moved(base, offset, (0.1, 0.0)).0 - 0.2).abs() < 1e-6);
    }

    #[test]
    fn a_node_cannot_be_dragged_off_the_canvas_and_does_not_bank_up_surplus() {
        let base = (0.5, 0.5);
        let jammed = moved(base, (0.0, 0.0), (99.0, 99.0));
        let at = placed(base, jammed);
        assert!(at.0 <= LIMIT + 1e-6 && at.1 <= LIMIT + 1e-6, "{at:?} past the band");
        let near = placed(base, moved(base, (0.0, 0.0), (-99.0, -99.0)));
        assert!(near.0 >= 0.0 && near.1 >= 0.0, "{near:?} off the near edge");
        // Jammed into the edge then nudged back, it moves on that nudge.
        let stepped = placed(base, moved(base, jammed, (-0.1, 0.0)));
        assert!(stepped.0 < at.0 - 1e-6, "{stepped:?} did not move back from {at:?}");
    }

    #[test]
    fn an_undragged_node_sits_exactly_where_the_file_put_it() {
        let base = (0.3, 0.4);
        assert_eq!(
            placed(base, offset_of(&BState::new(), BHostType::Shader, "Principled BSDF")),
            base,
            "no entry in state is no displacement"
        );
    }

    #[test]
    fn a_drags_grab_origin_is_taken_on_the_first_move_and_not_retaken() {
        let mut drag = BNodeDrag::new("Principled BSDF", (0.0, 0.0));
        assert_eq!(drag.grabbed, None, "the press cannot know the canvas");
        assert_eq!(*drag.grabbed.get_or_insert((0.4, 0.4)), (0.4, 0.4));
        assert_eq!(*drag.grabbed.get_or_insert((0.9, 0.9)), (0.4, 0.4), "measured from one origin");
    }
}
