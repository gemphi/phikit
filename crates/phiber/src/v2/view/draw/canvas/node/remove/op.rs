//! `NODE_OT_delete` (`editors/space_node/node_edit.cc`): the selected nodes
//! come out of the tree, as one rule on `OpKey::NodeDelete`.
//!
//! The value names the editor (`<host>`), so one table serves every node
//! editor — the same shape as `NodeSelect` and `NodeView`, dispatched through
//! the menu's `fire_host`. The rule reads the selection (`canvas/node/select`)
//! rather than taking names in the value, because Delete acts on whatever is
//! selected, exactly as Blender's does; it then clears the selection, since the
//! nodes it named are gone.

use super::{encode, removed_of};
use crate::v2::host::BHostType;
use crate::v2::state::rule::BRule;
use crate::v2::state::{BKey as _, BPath, BState, NodeKey, OpKey};

pub struct BNodeDeleteRule;

impl BRule for BNodeDeleteRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::NodeDelete.path()
    }

    fn after(&self, _path: &BPath, value: &str, s: &mut BState) {
        let Some(host) = BHostType::from_slug(value) else { return };
        let selection = super::super::select::selection_of(s, host);
        if selection.is_empty() {
            s.remove(OpKey::NodeDelete);
            return;
        }
        let mut removed = removed_of(s, host);
        for name in &selection.names {
            removed.add(name);
        }
        s.set(NodeKey::Removed(host), &encode(&removed));
        // The selection named nodes that are now gone, so it clears — Blender's
        // delete ends with nothing selected.
        s.set(NodeKey::Selected(host), "");
        s.remove(OpKey::NodeDelete);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::removed_of;
    use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph};
    use crate::phiphx::scene::{SceneCorners, SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
    use crate::v2::state::CtxKey;
    use crate::v2::view::draw::canvas::node::select;

    fn with_graph(names: &[&str]) -> BState {
        let nodes = names
            .iter()
            .enumerate()
            .map(|(i, n)| BlendNode { name: n.to_string(), x: i as f32 * 200.0, y: 0.0, width: 140.0, ..Default::default() })
            .collect();
        let mut material = SceneMaterial::neutral();
        material.graph = BlendNodeGraph { nodes, links: Vec::new() };
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
        s
    }

    fn fire(s: &mut BState) {
        BNodeDeleteRule.after(&OpKey::NodeDelete.path(), BHostType::Shader.slug(), s);
    }

    #[test]
    fn the_rule_answers_only_its_own_op() {
        assert!(BNodeDeleteRule.wants(&OpKey::NodeDelete.path()));
        assert!(!BNodeDeleteRule.wants(&OpKey::NodeSelect.path()));
        assert!(!BNodeDeleteRule.wants(&NodeKey::Removed(BHostType::Shader).path()));
    }

    #[test]
    fn delete_removes_the_selection_and_clears_it() {
        let mut s = with_graph(&["A", "B", "C"]);
        s.set(NodeKey::Selected(BHostType::Shader), &select::encode(&select::BNodeSelection::of(&["A", "C"])));
        fire(&mut s);
        let removed = removed_of(&s, BHostType::Shader);
        assert!(removed.holds("A") && removed.holds("C"), "the selected nodes are removed");
        assert!(!removed.holds("B"), "the unselected one is left");
        assert!(select::selection_of(&s, BHostType::Shader).is_empty(), "the selection clears");
        assert_eq!(s.get(&OpKey::NodeDelete.path()), None, "the op is spent");
        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn delete_with_nothing_selected_removes_nothing() {
        let mut s = with_graph(&["A", "B"]);
        fire(&mut s);
        assert!(removed_of(&s, BHostType::Shader).is_empty(), "nothing selected, nothing removed");
        assert_eq!(s.get(&OpKey::NodeDelete.path()), None, "and the op is still spent");
        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn an_unknown_editor_writes_nothing() {
        let mut s = with_graph(&["A"]);
        s.set(NodeKey::Selected(BHostType::Shader), &select::encode(&select::BNodeSelection::of(&["A"])));
        BNodeDeleteRule.after(&OpKey::NodeDelete.path(), "no-such-editor", &mut s);
        assert!(removed_of(&s, BHostType::Shader).is_empty());
        crate::v2::scene::source::set_loaded_shared(None);
    }
}
