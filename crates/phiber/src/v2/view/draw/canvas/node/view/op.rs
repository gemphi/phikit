//! The node editor's view operators, as one rule on `OpKey::NodeView`.
//!
//! Two verbs: `<editor>:all` is `NODE_OT_view_all` (Home) — the whole graph
//! framed in the window — and `<editor>:selected` is `NODE_OT_view_selected`
//! (Numpad .), the same framing over the selected nodes alone. The maths is
//! `frame::view_all`, ported and tested there; what lives here is the
//! operator, which is the thing that had been missing. The framing existed
//! and nothing could ask for it.
//!
//! Pan and zoom are NOT here. They are gestures — a held middle button and the
//! wheel — and a gesture writes `NodeKey::View` straight from the canvas, where
//! it already knows the pointer and the region (`wire/frame.rs`). An operator
//! would have to be handed both. Blender splits them the same way: `VIEW2D_OT_*`
//! are modal operators driven by the mouse, while `NODE_OT_view_all` is a
//! keystroke that computes a rectangle and sets it.
//!
//! The rule resolves the theme from state (`resolve_ui_style`, as
//! `app/op/render.rs` does) because a node's card size is a theme value and the
//! graph's bounds are measured in cards.

use super::super::BNode;
use super::super::plan::BCardSize;
use super::{frame, store};
use crate::v2::host::BHostType;
use crate::v2::state::rule::BRule;
use crate::v2::state::{BKey as _, BPath, BState, NodeKey, OpKey};

pub struct BNodeViewRule;

impl BNodeViewRule {
    /// A card's size in view units, from the theme in force.
    fn card_size(s: &BState) -> BCardSize {
        let style = crate::v2::theme::resolve_ui_style(crate::v2::theme::ui_style_defaults_for(s), s);
        BCardSize {
            header: style.dimensions.header_height,
            unit: style.dimensions.row_height,
            width: style.dimensions.channels_width,
        }
    }
}

impl BRule for BNodeViewRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::NodeView.path()
    }

    fn after(&self, _path: &BPath, value: &str, s: &mut BState) {
        let mut parts = value.splitn(2, ':');
        let Some(host) = parts.next().and_then(BHostType::from_slug) else { return };
        let what = parts.next().unwrap_or_default();
        let Some(graph) = BNode::graph_of(s) else { return };
        let size = Self::card_size(s);
        let next = match what {
            // `node_view_all_exec`: the union of every node's `draw_bounds`,
            // framed for the view as it stands. `view_all` refuses an empty
            // tree, so a material with no nodes leaves the view alone.
            "all" => {
                // Each node sized by its real, link-dependent height, so a graph
                // with unconnected vector inputs frames to its true extent.
                let wired = super::super::link::edits_of(s, host).apply(&graph.links);
                let cur = store::view_of(s, host, &graph.nodes, |n| size.height_of(n, &super::super::link::linked_inputs(&wired, &n.name)), size.width);
                let bounds = frame::graph_bounds(&graph.nodes, |n| size.height_of(n, &super::super::link::linked_inputs(&wired, &n.name)), size.width);
                frame::view_all(bounds, &cur, graph.nodes.len() == 1)
            }
            // `NODE_OT_view_selected`: the same framing over the selected nodes
            // alone. Nothing selected leaves the view where it is, as Blender's
            // reports "No nodes to view" rather than moving.
            "selected" => {
                let sel = super::super::select::selection_of(s, host);
                let chosen: Vec<_> = graph.nodes.iter().filter(|n| sel.holds(&n.name)).cloned().collect();
                if chosen.is_empty() {
                    return;
                }
                let wired = super::super::link::edits_of(s, host).apply(&graph.links);
                let cur = store::view_of(s, host, &graph.nodes, |n| size.height_of(n, &super::super::link::linked_inputs(&wired, &n.name)), size.width);
                let bounds = frame::graph_bounds(&chosen, |n| size.height_of(n, &super::super::link::linked_inputs(&wired, &n.name)), size.width);
                frame::view_all(bounds, &cur, chosen.len() == 1)
            }
            _ => return,
        };
        s.set(NodeKey::View(host), &store::encode(&next));
        s.remove(OpKey::NodeView);
    }
}

#[cfg(test)]
#[path = "op_tests.rs"]
mod tests;
