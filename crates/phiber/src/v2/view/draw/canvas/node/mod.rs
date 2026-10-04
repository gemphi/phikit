//! The node graph of the active object's material — the real one, out of the file.
//!
//! This drew two hardcoded cards, "Principled BSDF" and "Material Output", at two
//! hardcoded fractions, for every file ever opened. It had to: nothing parsed a
//! node tree into anything the editor could reach. `phiphx`'s `shade/graph.rs`
//! reads it now (`bNodeTree.nodes` → `bNode`, and `bNodeTree.links`), so this
//! draws what the material actually contains.
//!
//! The framing is Blender's now too. This used to FIT the whole graph to the
//! canvas on every frame, because there was no view to draw through — and three
//! separate divergences were recorded against that one absence: a dragged node's
//! displacement had to be stored in canvas fractions, a node could not be
//! dragged off the canvas, and the socket hit-test's zoom-compensated slack had
//! no zoom to compensate against. `view/` is Blender's `SpaceNode.view2d`
//! (`View2D.cur`, its zoom step and `NODE_OT_view_all`), the canvas pans and
//! zooms through it, and all three divergences are gone with the fit.
//!
//! What fitting the graph became is the framing the editor OPENS on
//! (`view/store.rs`), which is what `NODE_OT_view_all` is in Blender: one
//! framing among many, reachable again rather than imposed every frame.
//!
//! One divergence remains, and it is about text rather than geometry. A card is
//! a div, so its position and size are fractions of the canvas
//! (`BNodeView::to_fraction` — the region cancels, which is what makes a card
//! placeable before the canvas is measured). Fractions scale the card with the
//! zoom, but `text_size` wants pixels and the element tree has none, so a
//! node's title does not shrink with its card; below `set::LABEL_FLOOR` it is
//! dropped instead. Blender also stops drawing the label rather than shrinking
//! it to nothing, so the behaviour matches at small sizes and diverges in the
//! middle, where Blender's text is scaled and phiber's is not.

pub mod drag;
pub mod link;
pub mod paint;
pub mod pick;
pub mod plan;
pub mod select;
pub mod set;
pub mod socket;
pub mod view;
pub mod wire;

use super::super::ctx::BRenderCtx;
use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph};
use crate::v2::state::CtxKey;
use crate::v2::theme::palette::BColor;
use gpui::AnyElement;

pub struct BNode;

/// The order nodes are drawn in: the active node last, so it is on top.
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
/// disappears behind it.
///
/// The rank had two levels while phiber's selection was a single name, because
/// there was no active-versus-selected distinction to make. It has all three
/// now (`select::rank`), which matters as soon as a group is selected: the node
/// the next operator treats as its subject is the one drawn in front.
///
/// The sort is stable, so nodes of equal rank keep the tree's own order — which
/// is the file's, and is what `ui_order` starts out as.
pub(super) fn draw_order<'a>(
    nodes: &'a [BlendNode],
    selection: &select::BNodeSelection,
) -> Vec<&'a BlendNode> {
    let mut ordered: Vec<&BlendNode> = nodes.iter().collect();
    ordered.sort_by_key(|n| select::rank(n, selection));
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
    crate::v2::theme::class_color(
        crate::v2::theme::class_of(&node.idname, node.active_output()),
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
    fn the_opening_framing_puts_blenders_top_node_at_the_top() {
        // Blender's y runs UP; the canvas's runs down. A node Blender puts at
        // the top must come out at the top, or every graph opens upside down.
        // The fit used to do this flip itself; it happens once now, in
        // `BNodeView::to_fraction`, and this is the fact it has to keep.
        let nodes = vec![node("low", 0.0, 0.0), node("high", 100.0, 200.0)];
        let view = view::store::opening(&nodes, |_| 80.0, 240.0);
        let (low, high) = (view.to_fraction((0.0, 0.0)), view.to_fraction((100.0, 200.0)));
        assert!(low.0 < high.0, "the node further right in Blender is further right here");
        assert!(high.1 < low.1, "the node higher in Blender is higher here, not lower");
        for p in [low, high] {
            assert!((0.0..=1.0).contains(&p.0) && (0.0..=1.0).contains(&p.1), "{p:?}");
        }
    }

    #[test]
    fn the_selected_node_is_drawn_last_so_it_is_on_top() {
        // `compare_node_depth`: selection raises a node. Without it a dragged
        // card slides behind whatever it is dragged over.
        let nodes = vec![node("a", 0.0, 0.0), node("b", 1.0, 1.0), node("c", 2.0, 2.0)];
        let one = |name: &str| select::BNodeSelection::of(&[name]);
        let order: Vec<&str> =
            draw_order(&nodes, &one("a")).iter().map(|n| n.name.as_str()).collect();
        assert_eq!(order, vec!["b", "c", "a"], "the selected node is painted over the others");
        // And the ones that are not selected keep the tree's own order.
        let order: Vec<&str> =
            draw_order(&nodes, &one("b")).iter().map(|n| n.name.as_str()).collect();
        assert_eq!(order, vec!["a", "c", "b"]);
    }

    #[test]
    fn the_active_node_is_drawn_over_the_rest_of_its_group() {
        // Three levels, not two: the node the next operator acts on is the one
        // in front, which is only expressible now that a selection is a set.
        let nodes = vec![node("a", 0.0, 0.0), node("b", 1.0, 1.0), node("c", 2.0, 2.0)];
        let group = select::BNodeSelection::of(&["b", "a"]);
        let order: Vec<&str> =
            draw_order(&nodes, &group).iter().map(|n| n.name.as_str()).collect();
        assert_eq!(order, vec!["c", "a", "b"], "c behind, a selected, b active and in front");
    }

    #[test]
    fn with_nothing_selected_the_file_decides_what_is_raised() {
        // `NODE_SELECT` out of the file is what the editor opens on, so it is
        // what raises a node until the user has clicked anything.
        let mut nodes = vec![node("a", 0.0, 0.0), node("b", 1.0, 1.0)];
        nodes[0].flag = crate::phiphx::blend::shade::graph::NODE_SELECT;
        let order: Vec<&str> = draw_order(&nodes, &select::BNodeSelection::default())
            .iter()
            .map(|n| n.name.as_str())
            .collect();
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
