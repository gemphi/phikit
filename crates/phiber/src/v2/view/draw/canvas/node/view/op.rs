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
mod tests {
    use super::*;
    use super::super::BNodeView;

    #[test]
    fn the_rule_answers_only_its_own_op() {
        assert!(BNodeViewRule.wants(&OpKey::NodeView.path()));
        assert!(!BNodeViewRule.wants(&OpKey::TimeView.path()));
        assert!(!BNodeViewRule.wants(&NodeKey::View(BHostType::Shader).path()));
    }

    #[test]
    fn an_op_for_an_unknown_editor_or_verb_writes_nothing() {
        // A rule reads a string off the store, so it has to survive one that
        // means nothing — from an older session, or a hand-edited key.
        for value in ["", "nonsense:all", "shader:sideways", "shader"] {
            let mut s = BState::default();
            BNodeViewRule.after(&OpKey::NodeView.path(), value, &mut s);
            assert_eq!(s.get(&NodeKey::View(BHostType::Shader).path()), None, "{value:?}");
        }
    }

    #[test]
    fn with_no_file_loaded_home_leaves_the_view_alone() {
        // `graph_of` finds nothing without a loaded object, and a framing of
        // nothing would be a view of nothing.
        let mut s = BState::default();
        let before = store::encode(&BNodeView::default());
        s.set(NodeKey::View(BHostType::Shader), &before);
        BNodeViewRule.after(&OpKey::NodeView.path(), "shader:all", &mut s);
        assert_eq!(s.get(&NodeKey::View(BHostType::Shader).path()), Some(before.as_str()));
    }

    #[test]
    fn home_frames_the_graph_of_the_loaded_material() {
        // The positive case, and the one that matters: the three tests above
        // would all pass if `after` did nothing at all. This is the one that
        // fails if the operator is a no-op, which is what the rest of the
        // feature was missing — the framing existed and nothing invoked it.
        use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph};
        use crate::phiphx::scene::{SceneCorners, SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
        use crate::v2::state::CtxKey;

        let at = |x: f32, y: f32| BlendNode { name: format!("n{x}"), x, y, width: 140.0, ..Default::default() };
        // Far apart, and far from the default window, so a view that frames
        // them cannot be the view it started with.
        let nodes = vec![at(-3000.0, -2000.0), at(4000.0, 2500.0)];
        let mut material = SceneMaterial::neutral();
        material.graph = BlendNodeGraph { nodes: nodes.clone(), links: Vec::new() };
        let mesh = SceneMesh {
            vertices: vec![[0.0; 3]],
            indices: Vec::new(),
            normals: SceneNormals::PerFace(Vec::new()),
            corners: SceneCorners::default(),
        };
        let mut object = SceneObject::identity("Cube", mesh);
        object.materials = vec![material];
        let model = SceneModel { objects: vec![object], ..Default::default() };
        crate::v2::scene::source::set_loaded_shared(Some(std::sync::Arc::new(model)));

        let mut s = BState::default();
        s.set(CtxKey::Object, "Cube");
        BNodeViewRule.after(&OpKey::NodeView.path(), "shader:all", &mut s);

        let written = s
            .get(&NodeKey::View(BHostType::Shader).path())
            .expect("Home must write a view")
            .to_string();
        let framed = store::decode(&written).expect("and a readable one");
        // Every node is inside the framing, which is the whole claim of
        // `NODE_OT_view_all`.
        for node in &nodes {
            let seen = framed.to_fraction((node.x, node.y));
            assert!(
                (0.0..=1.0).contains(&seen.0) && (0.0..=1.0).contains(&seen.1),
                "{} at {:?} is outside the framing {framed:?}",
                node.name,
                seen,
            );
        }
        // And the op is consumed, so it does not re-fire on the next write.
        assert_eq!(s.get(&OpKey::NodeView.path()), None, "the op is spent");

        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn view_selected_frames_the_selected_nodes_alone() {
        // `NODE_OT_view_selected`: the framing follows the selection, so the
        // selected node lands inside and the unselected one need not.
        use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph};
        use crate::phiphx::scene::{SceneCorners, SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
        use crate::v2::state::{CtxKey, NodeKey};
        use crate::v2::view::draw::canvas::node::select;

        let at = |x: f32, y: f32| BlendNode { name: format!("n{x}"), x, y, width: 140.0, ..Default::default() };
        let near = at(-3000.0, -2000.0);
        let far = at(4000.0, 2500.0);
        let nodes = vec![near.clone(), far.clone()];
        let mut material = SceneMaterial::neutral();
        material.graph = BlendNodeGraph { nodes: nodes.clone(), links: Vec::new() };
        let mesh = SceneMesh {
            vertices: vec![[0.0; 3]],
            indices: Vec::new(),
            normals: SceneNormals::PerFace(Vec::new()),
            corners: SceneCorners::default(),
        };
        let mut object = SceneObject::identity("Cube", mesh);
        object.materials = vec![material];
        let model = SceneModel { objects: vec![object], ..Default::default() };
        crate::v2::scene::source::set_loaded_shared(Some(std::sync::Arc::new(model)));

        let mut s = BState::default();
        s.set(CtxKey::Object, "Cube");
        s.set(NodeKey::Selected(BHostType::Shader), &select::encode(&select::BNodeSelection::of(&[near.name.as_str()])));
        BNodeViewRule.after(&OpKey::NodeView.path(), "shader:selected", &mut s);

        let written = s.get(&NodeKey::View(BHostType::Shader).path()).expect("Frame Selected writes a view").to_string();
        let framed = store::decode(&written).expect("a readable one");
        let seen = framed.to_fraction((near.x, near.y));
        assert!(
            (0.0..=1.0).contains(&seen.0) && (0.0..=1.0).contains(&seen.1),
            "the selected node {:?} is inside its framing {framed:?}",
            seen,
        );
        // The unselected node is well outside: framing one node cannot also
        // frame one 7000 units away.
        let other = framed.to_fraction((far.x, far.y));
        assert!(other.0 > 1.0 || other.1 > 1.0, "the unselected node {other:?} is not framed");

        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn view_selected_with_nothing_selected_leaves_the_view_alone() {
        // Blender reports "No nodes to view" rather than moving.
        use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph};
        use crate::phiphx::scene::{SceneCorners, SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
        use crate::v2::state::{CtxKey, NodeKey};

        let mut material = SceneMaterial::neutral();
        material.graph = BlendNodeGraph {
            nodes: vec![BlendNode { name: "n".into(), x: 10.0, y: 10.0, width: 140.0, ..Default::default() }],
            links: Vec::new(),
        };
        let mesh = SceneMesh {
            vertices: vec![[0.0; 3]],
            indices: Vec::new(),
            normals: SceneNormals::PerFace(Vec::new()),
            corners: SceneCorners::default(),
        };
        let mut object = SceneObject::identity("Cube", mesh);
        object.materials = vec![material];
        let model = SceneModel { objects: vec![object], ..Default::default() };
        crate::v2::scene::source::set_loaded_shared(Some(std::sync::Arc::new(model)));

        let mut s = BState::default();
        s.set(CtxKey::Object, "Cube");
        let before = store::encode(&BNodeView::default());
        s.set(NodeKey::View(BHostType::Shader), &before);
        BNodeViewRule.after(&OpKey::NodeView.path(), "shader:selected", &mut s);
        assert_eq!(s.get(&NodeKey::View(BHostType::Shader).path()), Some(before.as_str()), "the view is left where it was");

        crate::v2::scene::source::set_loaded_shared(None);
    }
}
