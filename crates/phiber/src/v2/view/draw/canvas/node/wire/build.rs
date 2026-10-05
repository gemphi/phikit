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
    let nodes = &frame.graph.nodes;
    let moving: Vec<crate::v2::view::draw::canvas::node::drag::BNodeMove> = frame
        .located
        .iter()
        .filter(|(name, _)| {
            nodes.iter().find(|n| &n.name == name).is_some_and(|node| moves_with(node, nodes, frame.selection))
        })
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

/// Whether `node` travels when the selection is dragged: it is selected itself,
/// or a frame it is parented into (at any depth) is.
///
/// Blender's `get_transformed_nodes` adds the children of a selected frame to
/// the transform, so grabbing a frame moves the whole group. Walking the parent
/// chain is what carries a node inside a frame inside a selected frame; the
/// guard stops a malformed self-parent from looping.
pub(crate) fn moves_with(
    node: &crate::phiphx::blend::shade::graph::BlendNode,
    nodes: &[crate::phiphx::blend::shade::graph::BlendNode],
    selection: &select::BNodeSelection,
) -> bool {
    if select::rank_of_name(&node.name, selection, nodes) > 0 {
        return true;
    }
    let mut at = node.parent;
    let mut guard = 0;
    while let Some(addr) = at {
        guard += 1;
        if guard > 32 {
            break;
        }
        let Some(parent) = nodes.iter().find(|n| n.address == addr) else { break };
        if parent.frame() && select::rank_of_name(&parent.name, selection, nodes) > 0 {
            return true;
        }
        at = parent.parent;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::moves_with;
    use super::select::BNodeSelection;
    use crate::phiphx::blend::shade::graph::BlendNode;

    fn frame(address: u64) -> BlendNode {
        BlendNode { name: format!("f{address}"), idname: "NodeFrame".into(), address, ..Default::default() }
    }
    fn node(name: &str, address: u64, parent: Option<u64>) -> BlendNode {
        BlendNode { name: name.into(), address, parent, ..Default::default() }
    }

    #[test]
    fn dragging_a_frame_carries_the_nodes_parented_into_it() {
        // The whole point of a frame drag: the box moves and the group inside it
        // goes with it. A reader that moved only the selected node would leave
        // the frame's children behind.
        let f = frame(10);
        let inside = node("A", 1, Some(10));
        let outside = node("B", 2, None);
        let nodes = vec![f.clone(), inside.clone(), outside.clone()];
        let sel = BNodeSelection::of(&["f10"]);
        assert!(moves_with(&inside, &nodes, &sel), "a node in the selected frame moves");
        assert!(!moves_with(&outside, &nodes, &sel), "a node outside it does not");
        assert!(moves_with(&f, &nodes, &sel), "the frame itself moves");
    }

    #[test]
    fn the_parent_chain_is_followed_through_nested_frames() {
        // A node inside a frame inside the SELECTED frame still travels.
        let outer = frame(10);
        let mut inner = frame(20);
        inner.parent = Some(10);
        let deep = node("A", 1, Some(20));
        let nodes = vec![outer, inner, deep.clone()];
        assert!(moves_with(&deep, &nodes, &BNodeSelection::of(&["f10"])), "two frames deep still moves");
        // But not when it is a DIFFERENT frame that is selected.
        assert!(!moves_with(&deep, &nodes, &BNodeSelection::of(&["f99"])), "an unrelated frame moves nothing");
    }

    #[test]
    fn a_self_parented_frame_does_not_loop() {
        let mut f = frame(10);
        f.parent = Some(10);
        // Not selected, parented into itself: the guard returns rather than hangs.
        assert!(!moves_with(&f, &[f.clone()], &BNodeSelection::of(&["other"])));
    }
}
