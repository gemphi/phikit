//! The node editor's whole-graph selection operators, as one rule on
//! `OpKey::NodeSelect`.
//!
//! `NODE_OT_select_all` (`editors/space_node/node_select.cc`), the keystroke
//! half of selection: `<editor>:all` (A) turns every node on, `<editor>:none`
//! (Alt A) clears, and `<editor>:invert` (Ctrl I) swaps what is selected for
//! what is not. Picking one node, box- and drag-select are gestures — they
//! write `NodeKey::Selected` straight from the canvas, where the pointer is
//! known (`canvas/node/select`); these act on the graph as a whole, which a
//! gesture cannot, so they are the operator that was missing. The menu that
//! fired them (`menubar/editor`) used to fire the *viewport's* object select,
//! which read a 3D scene and left the node graph untouched.
//!
//! The value names the editor (`<host>:<verb>`), so one table serves every
//! node editor and each acts on its own selection — the same shape as
//! `NodeView` (Home), dispatched through the menu's `fire_host`.

use super::super::BNode;
use super::{BNodeSelection, encode, selection_of};
use crate::v2::host::BHostType;
use crate::v2::state::rule::BRule;
use crate::v2::state::{BKey as _, BPath, BState, NodeKey, OpKey};

pub struct BNodeSelectRule;

impl BRule for BNodeSelectRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::NodeSelect.path()
    }

    fn after(&self, _path: &BPath, value: &str, s: &mut BState) {
        let mut parts = value.splitn(2, ':');
        let Some(host) = parts.next().and_then(BHostType::from_slug) else { return };
        let what = parts.next().unwrap_or_default();
        let Some(graph) = BNode::graph_of(s) else { return };
        let names: Vec<&str> = graph.nodes.iter().map(|n| n.name.as_str()).collect();
        let cur = selection_of(s, host);
        let next = match what {
            // `SEL_SELECT`: every node on, the active one kept active.
            "all" => all_of(&names, &cur),
            // `SEL_DESELECT`.
            "none" => BNodeSelection::default(),
            // `SEL_INVERT`: the unselected become selected, in graph order.
            "invert" => BNodeSelection {
                names: names.iter().filter(|n| !cur.holds(n)).map(|n| n.to_string()).collect(),
            },
            _ => return,
        };
        s.set(NodeKey::Selected(host), &encode(&next));
        s.remove(OpKey::NodeSelect);
    }
}

/// Every node selected, with the current active one kept first so `select_all`
/// does not quietly move the subject the next operator acts on.
fn all_of(names: &[&str], cur: &BNodeSelection) -> BNodeSelection {
    let active = cur.active().filter(|a| names.contains(a));
    let mut ordered: Vec<String> = Vec::with_capacity(names.len());
    ordered.extend(active.map(str::to_string));
    ordered.extend(names.iter().filter(|n| Some(**n) != active).map(|n| n.to_string()));
    BNodeSelection { names: ordered }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph};
    use crate::phiphx::scene::{SceneCorners, SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
    use crate::v2::state::CtxKey;

    /// A loaded Cube whose one material has `names` as its node graph.
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

    fn selection(s: &BState) -> BNodeSelection {
        selection_of(s, BHostType::Shader)
    }

    fn fire(s: &mut BState, verb: &str) {
        BNodeSelectRule.after(&OpKey::NodeSelect.path(), &format!("shader:{verb}"), s);
    }

    #[test]
    fn the_rule_answers_only_its_own_op() {
        assert!(BNodeSelectRule.wants(&OpKey::NodeSelect.path()));
        assert!(!BNodeSelectRule.wants(&OpKey::NodeView.path()));
        assert!(!BNodeSelectRule.wants(&NodeKey::Selected(BHostType::Shader).path()));
    }

    #[test]
    fn all_selects_every_node_and_none_clears() {
        let mut s = with_graph(&["A", "B", "C"]);
        fire(&mut s, "all");
        let sel = selection(&s);
        for n in ["A", "B", "C"] {
            assert!(sel.holds(n), "{n} is selected by All");
        }
        fire(&mut s, "none");
        assert!(selection(&s).is_empty(), "None clears every node");
        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn all_keeps_the_active_node_active() {
        let mut s = with_graph(&["A", "B", "C"]);
        s.set(NodeKey::Selected(BHostType::Shader), &encode(&BNodeSelection::of(&["B"])));
        fire(&mut s, "all");
        assert_eq!(selection(&s).active(), Some("B"), "All does not move the subject");
        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn invert_swaps_the_selected_for_the_unselected() {
        let mut s = with_graph(&["A", "B", "C"]);
        s.set(NodeKey::Selected(BHostType::Shader), &encode(&BNodeSelection::of(&["A"])));
        fire(&mut s, "invert");
        let sel = selection(&s);
        assert!(!sel.holds("A"), "A was selected, now off");
        assert!(sel.holds("B") && sel.holds("C"), "B and C were off, now on");
        // And the op is spent, so it does not re-fire.
        assert_eq!(s.get(&OpKey::NodeSelect.path()), None);
        crate::v2::scene::source::set_loaded_shared(None);
    }

    #[test]
    fn an_unknown_editor_or_verb_writes_nothing() {
        for value in ["", "nonsense:all", "shader:sideways", "shader"] {
            let mut s = with_graph(&["A", "B"]);
            BNodeSelectRule.after(&OpKey::NodeSelect.path(), value, &mut s);
            assert_eq!(s.get(&NodeKey::Selected(BHostType::Shader).path()), None, "{value:?}");
        }
        crate::v2::scene::source::set_loaded_shared(None);
    }
}
