//! Framing a graph: what rectangle a node occupies, and the view that fits
//! them all.
//!
//! Split from `mod.rs`, which converts between view space and pixels. This is
//! the other direction — given the graph, what should the view BE — and it is
//! one operator (`NODE_OT_view_all`, the Home key) rather than part of every
//! frame's arithmetic.
//!
//! It was part of every frame in phiber, because fitting the graph was the only
//! framing there was.

use super::BNodeView;
use crate::phiphx::blend::shade::graph::BlendNode;

/// `BLI_rctf_scale(&cur_new, 1.1f)` — the padding "view all" leaves.
pub(super) const VIEW_ALL_PADDING: f32 = 1.1;

/// A node's `draw_bounds` in view space, Y up.
///
/// `bNode.location` is the node's **top-left** — the same fact the transform
/// code states (*"Use top-left corner as the transform origin for nodes"*) — so
/// the rectangle runs down and right from it.
///
/// `height_units` is the card's height from `socket::body_height` plus its
/// header, in widget units. The saved `bNode.height` is NOT used: Blender
/// recomputes it in `node_update_basis` from the socket layout every draw, so
/// the saved value is whatever the last session happened to leave there.
pub fn node_bounds(node: &BlendNode, height_units: f32, fallback_width: f32) -> (f32, f32, f32, f32) {
    // A width of zero means the file carried none for this type; the card's own
    // width is the honest stand-in rather than a zero-wide rectangle that no
    // box select could ever touch.
    let width = if node.width > 0.0 { node.width } else { fallback_width };
    (node.x, node.y - height_units, node.x + width, node.y)
}

/// `space_node_view_flag` — the framing that fits `bounds` for a view currently
/// shaped like `cur`. This is the Home key, and it is what phiber used to do on
/// every frame.
///
/// Three details, all Blender's:
///
/// * a **single** non-frame node that is already smaller on screen than the view
///   is centred and NOT zoomed to — *"for single non-frame nodes, don't zoom in,
///   just pan view, but do allow zooming out, this allows for big nodes to be
///   zoomed out"*;
/// * the aspect correction grows the short axis by a quarter of the needed
///   extent on each side, which is a loose fit with deliberate slack rather than
///   a tight letterbox;
/// * then 10% padding on everything (`BLI_rctf_scale(&cur_new, 1.1f)`).
pub fn view_all(bounds: (f32, f32, f32, f32), cur: &BNodeView, single_node: bool) -> BNodeView {
    let (old_w, old_h) = (cur.width(), cur.height());
    let (w, h) = (bounds.2 - bounds.0, bounds.3 - bounds.1);
    if w <= 0.0 || h <= 0.0 || old_h <= 0.0 {
        return *cur;
    }
    let centre = ((bounds.0 + bounds.2) / 2.0, (bounds.1 + bounds.3) / 2.0);
    let old_aspect = old_w / old_h;

    if single_node && old_w * old_h > w * h {
        // `BLI_rctf_resize`: keep the size, move the centre.
        return BNodeView {
            xmin: centre.0 - old_w / 2.0,
            xmax: centre.0 + old_w / 2.0,
            ymin: centre.1 - old_h / 2.0,
            ymax: centre.1 + old_h / 2.0,
        };
    }

    let mut next = BNodeView { xmin: bounds.0, ymin: bounds.1, xmax: bounds.2, ymax: bounds.3 };
    if old_aspect < w / h {
        let grown = w / old_aspect;
        next.ymin -= grown / 4.0;
        next.ymax += grown / 4.0;
    } else {
        let grown = h * old_aspect;
        next.xmin -= grown / 4.0;
        next.xmax += grown / 4.0;
    }
    scaled(&next, VIEW_ALL_PADDING)
}

/// `BLI_rctf_scale` — grow about the centre.
fn scaled(view: &BNodeView, by: f32) -> BNodeView {
    let (cx, cy) = ((view.xmin + view.xmax) / 2.0, (view.ymin + view.ymax) / 2.0);
    let (hw, hh) = (view.width() * by / 2.0, view.height() * by / 2.0);
    BNodeView { xmin: cx - hw, xmax: cx + hw, ymin: cy - hh, ymax: cy + hh }
}

/// Every node's rectangle together — what "view all" frames.
///
/// `NODE_OT_view_all` walks the tree accumulating `BLI_rctf_union` of each
/// node's `draw_bounds` (`node_view_all_exec` → `space_node_view_flag`), and
/// returns without moving the view when the tree is empty, which is the
/// `(0, 0, 0, 0)` `view_all` already refuses.
pub fn graph_bounds(
    nodes: &[BlendNode],
    height_of: impl Fn(&BlendNode) -> f32,
    fallback_width: f32,
) -> (f32, f32, f32, f32) {
    let mut bounds: Option<(f32, f32, f32, f32)> = None;
    for node in nodes {
        let (x0, y0, x1, y1) = node_bounds(node, height_of(node), fallback_width);
        bounds = Some(match bounds {
            None => (x0, y0, x1, y1),
            Some(b) => (b.0.min(x0), b.1.min(y0), b.2.max(x1), b.3.max(y1)),
        });
    }
    bounds.unwrap_or((0.0, 0.0, 0.0, 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::BNodeView;

    /// A view one view-unit per pixel wide on a 1000px canvas.
    fn unit_view() -> BNodeView {
        BNodeView { xmin: 0.0, ymin: 0.0, xmax: 1001.0, ymax: 600.0 }
    }

    fn node(x: f32, y: f32, width: f32) -> BlendNode {
        BlendNode { name: "n".into(), x, y, width, ..Default::default() }
    }

    #[test]
    fn a_nodes_bounds_run_down_and_right_from_its_location() {
        // `location` is the TOP-LEFT, the same fact the transform code states.
        let bounds = node_bounds(&node(100.0, 200.0, 140.0), 80.0, 999.0);
        assert_eq!(bounds, (100.0, 120.0, 240.0, 200.0));
    }

    #[test]
    fn a_node_with_no_saved_width_still_has_a_rectangle() {
        // A zero-wide node could never be touched by a box select, so the card's
        // own width stands in.
        let bounds = node_bounds(&node(0.0, 0.0, 0.0), 80.0, 240.0);
        assert_eq!(bounds.2 - bounds.0, 240.0);
    }

    #[test]
    fn view_all_frames_the_graph_with_padding_around_it() {
        let cur = unit_view();
        let bounds = (0.0, 0.0, 400.0, 300.0);
        let framed = view_all(bounds, &cur, false);
        assert!(framed.xmin < bounds.0 && framed.xmax > bounds.2, "{framed:?} does not contain it");
        assert!(framed.ymin < bounds.1 && framed.ymax > bounds.3);
    }

    #[test]
    fn view_all_grows_the_short_axis_rather_than_cropping_the_long_one() {
        // The aspect correction: whichever axis is short for the view's shape gets
        // a quarter of the needed extent added to each side. Loose, with slack,
        // not a tight letterbox.
        let cur = unit_view();
        // Much wider than the view's aspect.
        let wide = view_all((0.0, 0.0, 2000.0, 100.0), &cur, false);
        assert!(wide.height() > 100.0 * VIEW_ALL_PADDING, "the height grew: {}", wide.height());
        // Much taller.
        let tall = view_all((0.0, 0.0, 100.0, 2000.0), &cur, false);
        assert!(tall.width() > 100.0 * VIEW_ALL_PADDING, "the width grew: {}", tall.width());
    }

    #[test]
    fn view_all_on_one_small_node_centres_it_without_zooming_in() {
        // "for single non-frame nodes, don't zoom in, just pan view, but do allow
        // zooming out, this allows for big nodes to be zoomed out." Framing one
        // 140-wide node to fill a 1000px canvas would be a wall of card.
        let cur = unit_view();
        let bounds = (500.0, 300.0, 640.0, 380.0);
        let framed = view_all(bounds, &cur, true);
        assert!((framed.width() - cur.width()).abs() < 1e-3, "the zoom was kept");
        let centre = ((framed.xmin + framed.xmax) / 2.0, (framed.ymin + framed.ymax) / 2.0);
        assert!((centre.0 - 570.0).abs() < 1e-3, "centred on the node: {centre:?}");
        assert!((centre.1 - 340.0).abs() < 1e-3);
    }

    #[test]
    fn view_all_on_one_huge_node_does_zoom_out_to_it() {
        // The other half of the same condition: the exemption is only for a node
        // already smaller on screen than the view.
        let cur = unit_view();
        let huge = (0.0, 0.0, 9000.0, 7000.0);
        let framed = view_all(huge, &cur, true);
        assert!(framed.width() > cur.width(), "zoomed out to {}", framed.width());
    }

    #[test]
    fn view_all_on_an_empty_graph_leaves_the_view_alone() {
        // `if (tot == 0) return false` — the operator cancels and the view does
        // not move.
        let cur = unit_view();
        assert_eq!(view_all((0.0, 0.0, 0.0, 0.0), &cur, false), cur);
    }

    #[test]
    fn a_graphs_bounds_are_every_nodes_rectangle_together() {
        let nodes = vec![node(0.0, 100.0, 140.0), node(300.0, 400.0, 140.0)];
        let bounds = graph_bounds(&nodes, |_| 80.0, 240.0);
        assert_eq!(bounds, (0.0, 20.0, 440.0, 400.0));
        // An empty tree is the rectangle `view_all` already refuses to frame.
        assert_eq!(graph_bounds(&[], |_| 80.0, 240.0), (0.0, 0.0, 0.0, 0.0));
    }
}
