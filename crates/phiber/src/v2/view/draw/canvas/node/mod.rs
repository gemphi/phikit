//! The node graph of the active object's material — the real one, out of the file.
//!
//! This drew two hardcoded cards, "Principled BSDF" and "Material Output", at two
//! hardcoded fractions, for every file ever opened. It had to: nothing parsed a
//! node tree into anything the editor could reach. `phiphx`'s `shade/graph.rs`
//! reads it now (`bNodeTree.nodes` → `bNode`, and `bNodeTree.links`), so this
//! draws what the material actually contains.
//!
//! Two things here are Blender's and one is not. The nodes' positions are
//! Blender's own `bNode.location`, and their names and order come straight from
//! the tree. The **framing** is not: Blender gives the node editor a pannable,
//! zoomable view (`SpaceNode.view2d`) and draws nodes at their coordinates
//! through it, while this fits the whole graph to the canvas. Until the editor
//! can pan and zoom, fitting is the one framing that always shows the graph; a
//! 1:1 view with no way to move it would put most trees off-screen.

pub mod drag;
pub mod link;
pub mod paint;
pub mod pick;
pub mod plan;
pub mod set;
pub mod socket;
pub mod wire;

use super::super::ctx::BRenderCtx;
use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph};
use crate::v2::state::CtxKey;
use crate::v2::theme::palette::BColor;
use gpui::AnyElement;

/// How much of the canvas the graph is fitted into, and how far in it starts —
/// so nodes never touch the edges and a card still has room to its right.
const FIT: f32 = 0.72;
const INSET: f32 = 0.06;

pub struct BNode;

/// Where a node sits once the graph is fitted to the canvas, as fractions.
pub(super) struct BFitted {
    pub(super) x: f32,
    pub(super) y: f32,
}

/// The extent of a graph in Blender's own coordinates.
pub(super) struct BExtent {
    min_x: f32,
    min_y: f32,
    span_x: f32,
    span_y: f32,
}

impl BExtent {
    pub(super) fn of(nodes: &[BlendNode]) -> Self {
        let pick = |f: fn(&BlendNode) -> f32, best: fn(f32, f32) -> f32| {
            nodes.iter().map(f).fold(f32::NAN, best)
        };
        let (min_x, max_x) = (pick(|n| n.x, f32::min), pick(|n| n.x, f32::max));
        let (min_y, max_y) = (pick(|n| n.y, f32::min), pick(|n| n.y, f32::max));
        // One node, or a row of them, has no span on an axis; 1.0 keeps the
        // division honest and puts it at the inset corner.
        Self {
            min_x: if min_x.is_nan() { 0.0 } else { min_x },
            min_y: if min_y.is_nan() { 0.0 } else { min_y },
            span_x: if (max_x - min_x).abs() < 1.0 { 1.0 } else { max_x - min_x },
            span_y: if (max_y - min_y).abs() < 1.0 { 1.0 } else { max_y - min_y },
        }
    }

    /// Blender's y runs UP the canvas and the window's runs down, so the vertical
    /// share is flipped — a node Blender puts at the top is at the top here.
    pub(super) fn place(&self, node: &BlendNode) -> BFitted {
        BFitted {
            x: INSET + ((node.x - self.min_x) / self.span_x) * FIT,
            y: INSET + (1.0 - (node.y - self.min_y) / self.span_y) * FIT,
        }
    }
}

/// The order nodes are drawn in: the selected one last, so it is on top.
///
/// Blender's, from `node_draw.cc`. `tree_draw_order_calc_nodes` sorts by
/// `bNode.ui_order`, and `tree_draw_order_update` — called by every selection
/// operator in `node_select.cc` — re-sorts that order through
/// `compare_node_depth`, which ranks **active above selected above nothing**.
/// So in Blender selecting a node RAISES it, and `node_under_mouse_get` walks
/// the same order reversed, which makes the raised node the one a click finds.
///
/// phiber drew nodes in tree order, so a selected node could sit under an
/// unselected neighbour — and once a node can be dragged, that is the common
/// case rather than the odd one: you grab a card, drag it over another, and it
/// disappears behind it. With one selection there is no active-versus-selected
/// distinction to make, so the rank has two levels rather than three.
///
/// The sort is stable, so nodes that are not selected keep the tree's own order
/// — which is the file's, and is what `ui_order` starts out as.
pub(super) fn draw_order<'a>(nodes: &'a [BlendNode], selected: &str) -> Vec<&'a BlendNode> {
    let mut ordered: Vec<&BlendNode> = nodes.iter().collect();
    let raised = |n: &BlendNode| if selected.is_empty() { n.selected() } else { n.name == selected };
    ordered.sort_by_key(|n| raised(n));
    ordered
}

/// A node's header colour, by the class of node it is.
///
/// This used to be three colours chosen on the spot — the accent for anything
/// with "Output" in it, the status green for a shader, the X axis's red for a
/// texture. Blender has seventeen classes with theme values of their own
/// (`node_get_colorid`, `node_draw.cc:1388`), and they are the first thing
/// anyone reads off a graph. `theme::class_of` says which class a node is and
/// `theme::class_color` gives it Blender's hue at the theme's own header
/// lightness; both, and the reasoning for the split, are in
/// `theme/style/node_class/`.
///
/// `NODE_DO_OUTPUT` is why the whole node is passed rather than its idname: an
/// output node is only tinted when it is the output actually in use.
pub(super) fn header_color(node: &BlendNode, ctx: &BRenderCtx) -> BColor {
    let active_output = node.flag & crate::v2::theme::style::node_class::NODE_DO_OUTPUT != 0;
    crate::v2::theme::class_color(
        crate::v2::theme::class_of(&node.idname, active_output),
        ctx.style.colors.bg_control,
    )
}

impl BNode {
    /// The graph of the active object's first material, when a file is loaded.
    pub(super) fn graph(ctx: &BRenderCtx) -> Option<BlendNodeGraph> {
        let active = ctx.state.get(CtxKey::Object)?.to_string();
        crate::v2::scene::source::with_loaded(|loaded| {
            let model = loaded?;
            let object = model.objects.iter().find(|o| o.name == active)?;
            let graph = &object.materials.first()?.graph;
            (!graph.is_empty()).then(|| graph.clone())
        })
    }

    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        set::render(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(name: &str, x: f32, y: f32) -> BlendNode {
        BlendNode { name: name.into(), x, y, ..Default::default() }
    }

    #[test]
    fn a_graph_is_fitted_into_the_canvas_with_blenders_y_flipped() {
        // Blender's y runs UP; the window's runs down. A node Blender puts at the
        // top must come out at the top, or every graph opens upside down.
        let nodes = vec![node("low", 0.0, 0.0), node("high", 100.0, 200.0)];
        let extent = BExtent::of(&nodes);
        let (low, high) = (extent.place(&nodes[0]), extent.place(&nodes[1]));
        assert!(low.x < high.x, "the node further right in Blender is further right here");
        assert!(high.y < low.y, "the node higher in Blender is higher here, not lower");
        for p in [&low, &high] {
            assert!((0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.y), "{} {}", p.x, p.y);
        }
    }

    #[test]
    fn a_graph_with_one_node_does_not_divide_by_its_own_span() {
        // One node has no extent on either axis, so the span must not be zero.
        let nodes = vec![node("only", 42.0, -17.0)];
        let at = BExtent::of(&nodes).place(&nodes[0]);
        assert!(at.x.is_finite() && at.y.is_finite(), "{} {}", at.x, at.y);
        assert!((0.0..=1.0).contains(&at.x) && (0.0..=1.0).contains(&at.y));
    }

    #[test]
    fn the_selected_node_is_drawn_last_so_it_is_on_top() {
        // `compare_node_depth`: selection raises a node. Without it a dragged
        // card slides behind whatever it is dragged over.
        let nodes = vec![node("a", 0.0, 0.0), node("b", 1.0, 1.0), node("c", 2.0, 2.0)];
        let order: Vec<&str> = draw_order(&nodes, "a").iter().map(|n| n.name.as_str()).collect();
        assert_eq!(order, vec!["b", "c", "a"], "the selected node is painted over the others");
        // And the ones that are not selected keep the tree's own order.
        let order: Vec<&str> = draw_order(&nodes, "b").iter().map(|n| n.name.as_str()).collect();
        assert_eq!(order, vec!["a", "c", "b"]);
    }

    #[test]
    fn with_nothing_selected_the_file_decides_what_is_raised() {
        // `NODE_SELECT` out of the file is what the editor opens on, so it is
        // what raises a node until the user has clicked anything.
        let mut nodes = vec![node("a", 0.0, 0.0), node("b", 1.0, 1.0)];
        nodes[0].flag = crate::phiphx::blend::shade::graph::NODE_SELECT;
        let order: Vec<&str> = draw_order(&nodes, "").iter().map(|n| n.name.as_str()).collect();
        assert_eq!(order, vec!["b", "a"], "the file's selected node is raised");
    }

    #[test]
    fn a_node_is_titled_by_its_label_and_identified_by_its_type() {
        let mut n = node("Principled BSDF", 0.0, 0.0);
        n.idname = "ShaderNodeBsdfPrincipled".into();
        assert_eq!(n.title(), "Principled BSDF");
        n.label = "Skin".into();
        assert_eq!(n.title(), "Skin", "a labelled node is called by its label");
    }
}
