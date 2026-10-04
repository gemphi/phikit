//! What a selection means for draw order, and whether a box touches a card.
//!
//! Both answer questions ABOUT a selection rather than changing one, which is
//! why they sit apart from the click rules in `mod.rs`.

use super::BNodeSelection;
use crate::phiphx::blend::shade::graph::BlendNode;

/// Whether two rectangles touch — `BLI_rctf_isect`.
///
/// Each is `(left, top, right, bottom)` in canvas pixels, with the top above the
/// bottom numerically because the canvas's Y grows downward.
pub fn touches(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    a.0 <= b.2 && a.2 >= b.0 && a.1 <= b.3 && a.3 >= b.1
}

/// The draw-order rank of `node`: 0 behind, 1 selected, 2 active.
///
/// `compare_node_depth`'s *"One has a higher selection state (active >
/// selected > nothing)"*. A stable ascending sort by this puts the active node
/// in front, the rest of the selection over everything unselected, and leaves
/// nodes of equal rank in the file's own order — which is what `ui_order`
/// starts out as.
pub fn rank(node: &BlendNode, selection: &BNodeSelection) -> u8 {
    // With nothing selected in the editor, the file's own flags decide: it is
    // what the editor opens on.
    if selection.is_empty() {
        return match (node.active(), node.selected()) {
            (true, _) => 2,
            (false, true) => 1,
            _ => 0,
        };
    }
    match (selection.active() == Some(node.name.as_str()), selection.holds(&node.name)) {
        (true, _) => 2,
        (false, true) => 1,
        _ => 0,
    }
}

/// `rank` for a place that has a node's NAME rather than the node.
///
/// Same answer, looked up: the file's flags are only consulted when the editor
/// has no selection of its own, and that needs the node.
pub fn rank_of_name(name: &str, selection: &BNodeSelection, nodes: &[BlendNode]) -> u8 {
    match nodes.iter().find(|n| n.name == name) {
        Some(node) => rank(node, selection),
        None => 0,
    }
}
