//! Assembling one frame's gesture state.
//!
//! Split from `mod.rs` for the read budget (`level_structure`), and it is the
//! right seam anyway: `mod.rs` is what the pointer DOES, this is what one frame
//! hands it. `set::render` used to build `BWire` inline, which put the decision
//! about what the handlers need a long way from the handlers.

use super::{BLinking, BWire, link, select};
use super::super::super::super::ctx::BRenderCtx;
use crate::phiphx::blend::shade::graph::{BlendNodeGraph, BlendNodeLink};
use crate::v2::state::{BKey as _, NodeKey};
use gpui::{Bounds, Pixels};
use std::cell::Cell;
use std::rc::Rc;

/// What one frame hands the gesture layer, before `BWire` is assembled from it.
///
/// A struct rather than a dozen arguments, and borrowed rather than cloned,
/// because every field is something `set::render` has already computed for the
/// painter — this is the same frame read a second way, not a second frame.
pub struct BFrame<'a> {
    pub graph: &'a BlendNodeGraph,
    pub wired: Vec<BlendNodeLink>,
    pub placed: Vec<(String, (f32, f32))>,
    pub edits: link::BLinkEdits,
    pub carrying: Option<BLinking>,
    pub selection: &'a select::BNodeSelection,
    /// Every node's own `bNode.location`, in view units.
    pub located: &'a [(String, (f32, f32))],
    /// Every node's stored displacement, in view units.
    pub offsets: &'a [(String, (f32, f32))],
    pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    pub size: crate::v2::view::draw::canvas::node::plan::BCardSize,
    pub view: crate::v2::view::draw::canvas::node::view::BNodeView,
}

/// The gesture state for this frame, and what a drag of it would carry.
///
/// Assembled here rather than in `set::render` so that what the handlers need is
/// decided next to the handlers. The drag list is Blender's
/// `get_transformed_nodes`, evaluated ONCE when the transform begins rather than
/// per mouse move — which is also why it has to be gathered at render time: the
/// move handler cannot read the store.
pub fn for_frame(frame: BFrame<'_>, ctx: &BRenderCtx) -> (Rc<BWire>, Vec<crate::v2::view::draw::canvas::node::drag::BNodeMove>) {
    let moving: Vec<crate::v2::view::draw::canvas::node::drag::BNodeMove> = frame
        .located
        .iter()
        .filter(|(name, _)| select::rank_of_name(name, frame.selection, &frame.graph.nodes) > 0)
        .map(|(name, _)| crate::v2::view::draw::canvas::node::drag::BNodeMove {
            name: crate::v2::state::intern(name),
            from: frame.offsets.iter().find(|(n, _)| n == name).map(|(_, o)| *o).unwrap_or((0.0, 0.0)),
        })
        .collect();
    let wire = Rc::new(BWire {
        graph: frame.graph.clone(),
        wired: frame.wired,
        placed: frame.placed,
        edits: frame.edits,
        drag: frame.carrying,
        selection: frame.selection.clone(),
        box_from: ctx
            .state
            .get(&NodeKey::Boxing(ctx.host).path())
            .and_then(|text| text.split_once(','))
            .and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?))),
        handler: ctx.handler.clone(),
        host: ctx.host,
        bounds: frame.bounds,
        size: frame.size,
        view: frame.view,
    });
    (wire, moving)
}
