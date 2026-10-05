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

pub mod add;
pub mod drag;
pub mod frame;
pub mod link;
pub mod paint;
pub mod pick;
pub mod plan;
pub mod reroute;
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
    let hue = crate::v2::theme::class_color(
        crate::v2::theme::class_of(&node.idname, node.active_output()),
        ctx.style.colors.bg_control,
    );
    // A muted node reads as off: its header dims toward the card's body —
    // Blender's `get_color_blend_4f(TH_BACK, color_id, 0.4)`, mostly background
    // with a little class hue left so the class is still legible.
    if node.muted() {
        return dim_toward(hue, ctx.style.colors.bg_panel, MUTED_HEADER);
    }
    hue
}

/// The fraction of the class hue a muted header keeps; the rest is background
/// (`get_color_blend_4f`'s factor, Blender's 0.4).
const MUTED_HEADER: f32 = 0.4;

/// `a` blended toward `b`, keeping fraction `t` of `a`; alpha is `a`'s.
fn dim_toward(a: BColor, b: BColor, t: f32) -> BColor {
    let mix = |x: f32, y: f32| x + (y - x) * (1.0 - t);
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2), a.3)
}

/// The title ink, faded on a muted node (`BUT_INACTIVE`, the weight a muted
/// control takes — `effects.control_muted_alpha`).
pub(super) fn title_ink(node: &BlendNode, ctx: &BRenderCtx) -> BColor {
    let ink = ctx.style.colors.text;
    if node.muted() {
        (ink.0, ink.1, ink.2, ink.3 * ctx.style.effects.control_muted_alpha)
    } else {
        ink
    }
}

impl BNode {
    /// The graph of the active object's first material, when a file is loaded,
    /// with this host's added nodes merged in (`add::BNodeAdditions`). The
    /// overlay is empty until the Add menu writes it, so this is the file's own
    /// tree until then.
    pub(super) fn graph(ctx: &BRenderCtx) -> Option<BlendNodeGraph> {
        let mut graph = Self::graph_of(ctx.state)?;
        graph.nodes = add::additions_of(ctx.state, ctx.host).apply(&graph.nodes);
        Some(graph)
    }

    /// The same, from the store alone — what a rule has (`view/op.rs`).
    ///
    /// It never needed the render context: the active object is a state key and
    /// the tree comes from the loaded file, so the `ctx` in the signature above
    /// was only ever there because its one caller had one.
    pub(crate) fn graph_of(state: &crate::v2::state::BState) -> Option<BlendNodeGraph> {
        let active = state.get(CtxKey::Object)?.to_string();
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
mod tests;
