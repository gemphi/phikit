//! `NODE_OT_duplicate` (Shift D): the selected nodes copied, as one rule on
//! `OpKey::NodeDuplicate`.
//!
//! A duplicate is an addition — the same overlay a fresh node from the Add menu
//! writes (`add/`). Each selected node is re-added by its type, uniquely named
//! off the original's stem (`Emission.001` → `Emission.002`, Blender's
//! `nodeUniqueName`), offset a little so it does not sit exactly on top, and its
//! socket-value edits are carried onto the copy through the value overlay
//! (`edit/`) — the same place a hand edit to any node lives, so a duplicate
//! keeps the values the original was given. Blender then selects the copies and
//! deselects the originals, which is what lets Shift D G drag the new ones.
//!
//! The overlay rebuilds a node from the catalog, so a node whose type the
//! catalog does not carry cannot be duplicated and is skipped — the same limit
//! `BNodeAdditions::apply` already has, named here rather than hidden. Internal
//! links among a duplicated set are not copied yet; that waits on a links-by-set
//! copy the link overlay does not express.

use super::{BNodeAddition, additions_of, encode, find, unique_name};
use crate::v2::host::BHostType;
use crate::v2::state::rule::BRule;
use crate::v2::state::{BKey as _, BPath, BState, NodeKey, OpKey};

pub struct BNodeDuplicateRule;

/// How far a duplicate sits from its original, in view units — enough to read
/// as a second card rather than a thick border on the first.
const OFFSET: (f32, f32) = (30.0, -30.0);

impl BRule for BNodeDuplicateRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::NodeDuplicate.path()
    }

    fn after(&self, _path: &BPath, value: &str, s: &mut BState) {
        let done = |s: &mut BState| s.remove(OpKey::NodeDuplicate);
        let Some(host) = BHostType::from_slug(value) else { return };
        let selection = super::super::select::selection_of(s, host);
        let Some(file) = super::super::BNode::graph_of(s) else { return done(s) };
        if selection.is_empty() {
            return done(s);
        }
        // The graph as it stands — the file's nodes, the additions merged in,
        // the deletions taken out — so a duplicate reads the same node the eye
        // sees, added or not.
        let mut nodes = additions_of(s, host).apply(&file.nodes);
        super::super::remove::removed_of(s, host).apply(&mut nodes);

        let mut additions = additions_of(s, host);
        let mut values = super::super::edit::edits_of(s, host);
        let mut taken: Vec<String> = nodes.iter().map(|n| n.name.clone()).collect();
        let mut copies: Vec<String> = Vec::new();

        for name in &selection.names {
            let Some(node) = nodes.iter().find(|n| &n.name == name) else { continue };
            // A type the catalog cannot rebuild would be dropped by `apply`, so
            // it is not pretended to have been duplicated.
            if find(&node.idname).is_none() {
                continue;
            }
            let fresh = unique_name(stem(&node.name), &taken.iter().map(String::as_str).collect::<Vec<_>>());
            additions.added.push(BNodeAddition {
                idname: node.idname.clone(),
                name: fresh.clone(),
                at: (node.x + OFFSET.0, node.y + OFFSET.1),
                link: None,
            });
            for edit in values.edits.clone().iter().filter(|e| &e.node == name) {
                values.set(&fresh, &edit.socket, edit.value.clone());
            }
            taken.push(fresh.clone());
            copies.push(fresh);
        }

        if copies.is_empty() {
            return done(s);
        }
        s.set(NodeKey::Added(host), &encode(&additions.added));
        s.set(NodeKey::Values(host), &super::super::edit::encode(&values));
        // The copies are selected, the originals are not — Blender ends a
        // duplicate with the new nodes ready to move.
        s.set(NodeKey::Selected(host), &super::super::select::encode(&super::super::select::BNodeSelection { names: copies }));
        done(s);
    }
}

/// A name with a trailing `.NNN` cut off, so a copy counts up from the same
/// stem the original does (`Emission.001` and `Emission.002` are both
/// `Emission`) rather than nesting (`Emission.001.001`).
fn stem(name: &str) -> &str {
    name.rsplit_once('.')
        .filter(|(_, suffix)| suffix.len() == 3 && suffix.bytes().all(|b| b.is_ascii_digit()))
        .map_or(name, |(base, _)| base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::{edit, remove, select};
    use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph};
    use crate::phiphx::blend::shade::socket::BSocketDefault;
    use crate::phiphx::scene::{SceneCorners, SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
    use crate::v2::state::CtxKey;

    /// A loaded Cube whose material graph is `idnames` (one node each, named by
    /// its idname for the test's convenience).
    fn with_graph(nodes: &[(&str, &str)]) -> BState {
        let graph_nodes = nodes
            .iter()
            .enumerate()
            .map(|(i, (idname, name))| BlendNode {
                idname: idname.to_string(),
                name: name.to_string(),
                x: i as f32 * 200.0,
                y: 0.0,
                width: 140.0,
                ..Default::default()
            })
            .collect();
        let mut material = SceneMaterial::neutral();
        material.graph = BlendNodeGraph { nodes: graph_nodes, links: Vec::new() };
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

    fn select_names(s: &mut BState, names: &[&str]) {
        s.set(NodeKey::Selected(BHostType::Shader), &select::encode(&select::BNodeSelection::of(names)));
    }

    fn fire(s: &mut BState) {
        BNodeDuplicateRule.after(&OpKey::NodeDuplicate.path(), BHostType::Shader.slug(), s);
    }

    /// A catalog idname, so the duplicate is not dropped as unrebuildable.
    const KNOWN: &str = "ShaderNodeEmission";

    #[test]
    fn the_rule_answers_only_its_own_op() {
        assert!(BNodeDuplicateRule.wants(&OpKey::NodeDuplicate.path()));
        assert!(!BNodeDuplicateRule.wants(&OpKey::NodeDelete.path()));
    }

    #[test]
    fn duplicating_adds_a_uniquely_named_copy_and_selects_it() {
        let mut s = with_graph(&[(KNOWN, "Emission")]);
        select_names(&mut s, &["Emission"]);
        fire(&mut s);
        let added = additions_of(&s, BHostType::Shader).added;
        assert_eq!(added.len(), 1, "one copy added");
        assert_eq!(added[0].idname, KNOWN);
        assert_eq!(added[0].name, "Emission.001", "named off the original's stem");
        assert_ne!(added[0].at, (0.0, 0.0), "offset, not on top of the original");
        let sel = select::selection_of(&s, BHostType::Shader);
        assert_eq!(sel.names, ["Emission.001"], "the copy is selected, the original is not");
        assert_eq!(s.get(&OpKey::NodeDuplicate.path()), None, "the op is spent");
        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn a_copy_carries_the_original_s_socket_values() {
        let mut s = with_graph(&[(KNOWN, "Emission")]);
        let mut edits = edit::BValueEdits::default();
        edits.set("Emission", "Strength", BSocketDefault::Float(2.5));
        s.set(NodeKey::Values(BHostType::Shader), &edit::encode(&edits));
        select_names(&mut s, &["Emission"]);
        fire(&mut s);
        let after = edit::edits_of(&s, BHostType::Shader);
        assert_eq!(
            after.get("Emission.001", "Strength"),
            Some(&BSocketDefault::Float(2.5)),
            "the copy keeps the value the original was given",
        );
        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn a_node_the_catalog_cannot_rebuild_is_skipped() {
        let mut s = with_graph(&[("ShaderNodeMadeUp", "Mystery")]);
        select_names(&mut s, &["Mystery"]);
        fire(&mut s);
        assert!(additions_of(&s, BHostType::Shader).added.is_empty(), "no unrebuildable copy");
        // The selection is left as it was, since nothing was duplicated.
        assert_eq!(select::selection_of(&s, BHostType::Shader).names, ["Mystery"]);
        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn duplicating_a_duplicate_counts_up_from_the_stem() {
        let mut s = with_graph(&[(KNOWN, "Emission"), (KNOWN, "Emission.001")]);
        select_names(&mut s, &["Emission.001"]);
        fire(&mut s);
        let names: Vec<_> = additions_of(&s, BHostType::Shader).added.iter().map(|a| a.name.clone()).collect();
        assert_eq!(names, ["Emission.002"], "not Emission.001.001");
        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn deleted_nodes_are_not_duplicated() {
        let mut s = with_graph(&[(KNOWN, "Emission")]);
        s.set(NodeKey::Removed(BHostType::Shader), &remove::encode(&remove::BNodeRemovals::of(&["Emission"])));
        select_names(&mut s, &["Emission"]);
        fire(&mut s);
        assert!(additions_of(&s, BHostType::Shader).added.is_empty(), "a removed node is not on the merged graph to copy");
        crate::v2::scene::source::set_loaded_shared(None);
    }
}
