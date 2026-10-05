//! `node_update_collapsed` — a node folded with H.
//!
//! Split from `mod.rs`, which is `node_update_basis`: the open card's layout,
//! where the socket walk IS the height calculation. A collapsed node is laid out
//! a different way entirely, not a special case of that walk — it is a pill of a
//! height set by whichever socket list is longer, with each list centred on it,
//! and its rectangle is CENTRED on the node's location where an open card hangs
//! below it.
//!
//! phiber ignored the flag, so every node a saved file had folded was drawn as a
//! full card: the wrong shape, the wrong height, sockets in the wrong places, and
//! a hit-test that refused the top row of it (`pick/`, which skips its header
//! exclusion for a collapsed node exactly as Blender does).

use super::{BSocketRow, BSocketSide, HALF};
use crate::phiphx::blend::shade::graph::BlendNode;
use crate::phiphx::blend::shade::socket::BlendSocket;

/// `BASIS_RAD` — `0.2 * widget_unit`, the card's corner radius, which is also
/// the padding a collapsed node's pill carries above and below its sockets.
pub const COLLAPSED_RAD: f32 = 0.2;

/// `node_update_collapsed`'s `offset`: `NODE_DY * -0.5`, with the reason given
/// in Blender — *"This offset for Y values keeps the text in the same spot as
/// in non-collapsed nodes."*
const COLLAPSED_OFFSET: f32 = -HALF;

/// How many of `sockets` are drawn.
fn shown(sockets: &[BlendSocket]) -> usize {
    sockets.iter().filter(|s| s.visible()).count()
}

/// A collapsed node's full height in units — `node_update_collapsed`:
///
/// ```text
/// dy     = NODE_DY * 0.5
/// height = dy * max(totin, totout, 2) + BASIS_RAD * 2
/// ```
///
/// The `max(.., 2)` is Blender's floor: a pill is never shorter than two socket
/// rows, so a node with one socket is still a pill and not a dot.
pub fn collapsed_height(node: &BlendNode) -> f32 {
    let widest = shown(&node.inputs).max(shown(&node.outputs)).max(2);
    HALF * widest as f32 + COLLAPSED_RAD * 2.0
}

/// A collapsed node's top edge, as an offset from `bNode.location.y` in view
/// space (**Y up**).
///
/// A basis node's `draw_bounds.ymax` IS its location, so it hangs downward from
/// it. A collapsed node's is `loc.y + height * 0.5 + offset`, so its pill is
/// CENTRED on the location instead — which means a node folded with H does not
/// keep its top edge, and a reader that assumed it did would draw every
/// collapsed node half a pill too low.
pub fn collapsed_top(node: &BlendNode) -> f32 {
    collapsed_height(node) * 0.5 + COLLAPSED_OFFSET
}

/// A collapsed node's sockets: each list centred on the pill, inputs on the
/// left edge and outputs on the right.
///
/// Blender places the outputs from `loc.y + dy * (totout - 1) * 0.5 + offset`
/// stepping DOWN by `dy`, and the inputs the same way with their own count. Each
/// list is centred independently, so a node with one input and three outputs has
/// its single input level with the middle output.
///
/// `y` here is measured from the pill's TOP, because that is what
/// `BNodeCard::socket_at` adds to — a collapsed card carries no header, so its
/// `header` is zero and the rows start at its own top edge.
pub fn collapsed_rows(node: &BlendNode) -> Vec<BSocketRow> {
    let height = collapsed_height(node);
    let mut out = Vec::new();
    for (sockets, side) in [(&node.outputs, BSocketSide::Output), (&node.inputs, BSocketSide::Input)] {
        let total = shown(sockets);
        // The first of this list, below the top: half the pill, less half the
        // list's own extent. The `offset` cancels — it moves the pill's top and
        // the sockets together — which is why it is not in this line.
        let first = height / 2.0 - HALF * (total.saturating_sub(1)) as f32 / 2.0;
        let mut placed = 0;
        for socket in sockets.iter().filter(|s| s.visible()) {
            out.push(BSocketRow {
                identifier: socket.identifier.clone(),
                name: socket.name.clone(),
                kind: socket.kind,
                side,
                y: first + HALF * placed as f32,
                // A collapsed node draws its sockets as dots only — no rows to
                // hold a value widget in.
                default: None,
                linked: false,
            });
            placed += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::rows;
    use super::*;
    use crate::phiphx::blend::shade::socket::BlendSocket;

    fn sock(identifier: &str, kind: i32, flag: i32) -> BlendSocket {
        BlendSocket { identifier: identifier.into(), name: identifier.into(), kind, flag, ..Default::default() }
    }

    fn node(inputs: Vec<BlendSocket>, outputs: Vec<BlendSocket>) -> BlendNode {
        BlendNode { inputs, outputs, ..Default::default() }
    }

    fn body_height(node: &BlendNode) -> f32 {
        super::super::body_height(node)
    }

    // ── collapsed nodes (`node_update_collapsed`) ───────────────────────────────

    /// The same node, folded with H.
    fn folded(inputs: Vec<BlendSocket>, outputs: Vec<BlendSocket>) -> BlendNode {
        BlendNode {
            inputs,
            outputs,
            flag: crate::phiphx::blend::shade::graph::NODE_COLLAPSED,
            ..Default::default()
        }
    }

    fn socks(n: usize) -> Vec<BlendSocket> {
        (0..n).map(|i| sock(&format!("s{i}"), 2, 0)).collect()
    }

    #[test]
    fn a_pill_is_never_shorter_than_two_socket_rows() {
        // `dy * max(totin, totout, 2) + BASIS_RAD * 2`. The floor of 2 is
        // Blender's: a node with one socket is still a pill, not a dot.
        let two = HALF * 2.0 + COLLAPSED_RAD * 2.0;
        assert_eq!(collapsed_height(&folded(Vec::new(), Vec::new())), two, "no sockets at all");
        assert_eq!(collapsed_height(&folded(socks(1), Vec::new())), two, "one socket");
        assert_eq!(collapsed_height(&folded(socks(2), socks(1))), two, "two is still the floor");
        // Past the floor it is the WIDER list that sets the height, not the sum.
        assert_eq!(collapsed_height(&folded(socks(1), socks(5))), HALF * 5.0 + COLLAPSED_RAD * 2.0);
        assert_eq!(collapsed_height(&folded(socks(5), socks(1))), HALF * 5.0 + COLLAPSED_RAD * 2.0);
    }

    #[test]
    fn a_pill_is_centred_on_its_location_where_a_card_hangs_below_it() {
        // The difference a reader has to know: `draw_bounds.ymax` is the location
        // for a basis node and `loc.y + height * 0.5 + offset` for a collapsed one.
        // Getting this wrong draws every folded node half a pill out of place.
        let n = folded(socks(2), socks(2));
        let height = collapsed_height(&n);
        assert_eq!(collapsed_top(&n), height / 2.0 - HALF);
        // A basis node has no such offset at all.
        assert_eq!(collapsed_top(&folded(socks(6), socks(6))), collapsed_height(&folded(socks(6), socks(6))) / 2.0 - HALF);
    }

    #[test]
    fn each_list_is_centred_on_the_pill_independently() {
        // One input and three outputs: the input sits level with the MIDDLE
        // output, because each list is centred on its own count.
        let n = folded(socks(1), socks(3));
        let rows = rows(&n);
        let outs: Vec<f32> = rows.iter().filter(|r| r.side == BSocketSide::Output).map(|r| r.y).collect();
        let ins: Vec<f32> = rows.iter().filter(|r| r.side == BSocketSide::Input).map(|r| r.y).collect();
        assert_eq!(outs.len(), 3);
        assert_eq!(ins.len(), 1);
        assert!((ins[0] - outs[1]).abs() < 1e-6, "input at {:?}, middle output at {}", ins, outs[1]);
        // Stepping down by `dy`, and symmetric about the pill's middle.
        assert!((outs[1] - outs[0] - HALF).abs() < 1e-6);
        let middle = collapsed_height(&n) / 2.0;
        assert!((outs[0] + outs[2] - 2.0 * middle).abs() < 1e-6, "{outs:?} is not centred on {middle}");
    }

    #[test]
    fn a_folded_nodes_sockets_are_on_the_pills_two_edges() {
        let rows = rows(&folded(socks(1), socks(1)));
        assert_eq!(rows.iter().filter(|r| r.side == BSocketSide::Input).count(), 1);
        assert_eq!(rows.iter().filter(|r| r.side == BSocketSide::Output).count(), 1);
        // Every row sits inside the pill, which is what makes `socket_at` land on it.
        let height = collapsed_height(&folded(socks(1), socks(1)));
        for row in &rows {
            assert!((0.0..=height).contains(&row.y), "{} is outside a pill {height} tall", row.y);
        }
    }

    #[test]
    fn a_hidden_socket_takes_no_room_in_a_pill_either() {
        // `socket->is_visible()` gates the count in `node_update_collapsed` exactly
        // as it gates the walk in `node_update_basis`.
        use crate::phiphx::blend::shade::socket::{SOCK_HIDDEN, SOCK_UNAVAIL};
        let mut outputs = socks(4);
        outputs[1].flag = SOCK_HIDDEN;
        outputs[2].flag = SOCK_UNAVAIL;
        let n = folded(Vec::new(), outputs);
        assert_eq!(collapsed_height(&n), HALF * 2.0 + COLLAPSED_RAD * 2.0, "two shown, so the floor");
        assert_eq!(rows(&n).len(), 2, "and only the two shown are placed");
    }

    #[test]
    fn folding_a_node_changes_which_layout_answers() {
        // `rows` and `body_height` both dispatch, so every caller — the painter,
        // the hit-test, the card's own div — follows the flag without being told.
        let open = node(socks(3), socks(3));
        let shut = folded(socks(3), socks(3));
        assert_ne!(body_height(&open), body_height(&shut));
        assert_eq!(body_height(&shut), collapsed_height(&shut), "no header, no body: the pill is the height");
        assert_eq!(rows(&shut), collapsed_rows(&shut));
    }
}
