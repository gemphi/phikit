//! Object ▸ Parent (Ctrl P): the selected objects become children of the
//! active one, and Object ▸ Parent ▸ Clear (Alt P) takes a parent off.
//!
//! Ported from Blender `editors/object/object_relations.cc`:
//! `parent_set_exec` and `parent_clear_exec`. phiber keeps parent links as an
//! overlay (`SceneKey::Parented`, `BSceneLinks::parented`), so Set is one
//! `BSceneLinks::parent(child, active)` per selected child and Clear is one
//! `unparent(child)` per selected object — the overlay form of `ob->parent =
//! active_object` and `ob->parent = NULL`. The active object is itself not
//! parented to itself (`parent_set_exec` skips `par == ob`), and a selection
//! with no active is a no-op (same as Blender, which returns `OPERATOR_CANCELLED`
//! when `CTX_data_active_object` is null).
//!
//! Recorded divergences:
//! - Only `PAR_OBJECT` is wired. The ten typed parent kinds
//!   (`PAR_ARMATURE_NAME`, `PAR_BONE`, `PAR_VERTEX`, …) need rig support that
//!   phiber does not yet have — they come with it.
//! - Keep-transform (`CLEAR_PARENT_KEEP_TRANSFORM`,
//!   `BKE_object_apply_mat4(ob->object_to_world(), true, false)`) is a second
//!   step on top of the plain clear, which also rewrites the child's local
//!   matrix to preserve its world. It is a separate op that writes a
//!   `BTransform` beside the overlay change; left for its own turn.

use super::links::BSceneLinks;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, rule::BRule};

pub struct BObjectParentRule;

impl BRule for BObjectParentRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::ObjectParent.path()
    }

    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        s.remove(OpKey::ObjectParent);
        let Some(model) = crate::v2::scene::source::loaded_shared() else { return };
        let selected = model.selection.selected.clone();
        let active = s.get(CtxKey::Object).map(str::to_string);
        let mut links = BSceneLinks::now(s);
        match value {
            // `PAR_OBJECT`: every selected child but the parent itself takes
            // the active object as its parent.
            "set" => {
                let Some(parent) = active else { return };
                for child in &selected {
                    if child != &parent {
                        links.parent(child, &parent);
                    }
                }
            }
            // `CLEAR_PARENT_ALL`: every selected object loses its parent.
            "clear" => {
                for child in &selected {
                    links.unparent(child);
                }
            }
            _ => return,
        }
        links.write(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::scene::{SceneCorners, SceneMesh, SceneModel, SceneNormals, SceneObject};
    use crate::v2::state::SceneKey;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    fn mesh() -> SceneMesh {
        SceneMesh {
            vertices: vec![[0.0; 3]],
            indices: Vec::new(),
            normals: SceneNormals::PerFace(Vec::new()),
            corners: SceneCorners::default(),
        }
    }

    fn with(selected: &[&str], active: &str, parented: &[(&str, &str)]) -> BState {
        let objects = ["Cube", "Camera", "Light"].iter().map(|n| SceneObject::identity(*n, mesh())).collect();
        let mut model = SceneModel { objects, ..Default::default() };
        model.selection.selected = selected.iter().map(|s| s.to_string()).collect();
        crate::v2::scene::source::set_loaded_scene(Some(model));
        let mut s = BState::new();
        s.set(CtxKey::Object, active);
        let mut links = BSceneLinks::now(&s);
        for (c, p) in parented {
            links.parent(c, p);
        }
        links.write(&mut s);
        s
    }

    fn parents(s: &BState) -> Vec<(String, String)> {
        BSceneLinks::now(s).parented
    }

    fn fire(s: &mut BState, verb: &str) {
        apply_action(s, &BAction::Set(OpKey::ObjectParent.path(), verb.to_string()));
    }

    #[test]
    fn set_parents_the_selected_to_the_active_but_not_the_active_to_itself() {
        let mut s = with(&["Cube", "Camera"], "Cube", &[]);
        fire(&mut s, "set");
        // Blender's `parent_set_exec` loops the selected and skips `par == ob`.
        assert_eq!(parents(&s), [("Camera".to_string(), "Cube".to_string())]);
        assert_eq!(s.get(&OpKey::ObjectParent.path()), None, "the op is spent");
        crate::v2::scene::source::clear_loaded_scene();
    }

    #[test]
    fn set_replaces_a_previous_parent_rather_than_stacking() {
        let mut s = with(&["Camera"], "Cube", &[("Camera", "Light")]);
        fire(&mut s, "set");
        assert_eq!(parents(&s), [("Camera".to_string(), "Cube".to_string())]);
        crate::v2::scene::source::clear_loaded_scene();
    }

    #[test]
    fn clear_takes_the_parent_off_the_selected_and_keeps_other_links() {
        // Camera parented to Cube; selecting Camera and firing clear removes
        // only that entry — a Light also parented to Cube stays.
        let mut s = with(&["Camera"], "Cube", &[("Camera", "Cube"), ("Light", "Cube")]);
        fire(&mut s, "clear");
        assert_eq!(parents(&s), [("Light".to_string(), "Cube".to_string())]);
        crate::v2::scene::source::clear_loaded_scene();
    }

    #[test]
    fn set_without_an_active_does_nothing() {
        let model = SceneModel { objects: vec![SceneObject::identity("Cube", mesh())], ..Default::default() };
        crate::v2::scene::source::set_loaded_scene(Some(model));
        let mut s = BState::new();
        let before = s.get(SceneKey::Parented).unwrap_or("").to_string();
        fire(&mut s, "set");
        assert_eq!(s.get(SceneKey::Parented).unwrap_or("").to_string(), before, "nothing changed");
        crate::v2::scene::source::clear_loaded_scene();
    }
}
