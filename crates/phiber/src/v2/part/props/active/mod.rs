//! The active object's own values in the Properties editor — Blender's
//! `context.object` behind the Object and Material tabs. Structure: the
//! rule that follows the active object, and the entry the loader calls.
//! What is written, and from which field, is `set.rs`.

pub mod set;

use crate::phiphx::scene::SceneModel;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, rule::BRule};

/// `ctx.object` → the Object and Material tabs, as Blender redraws the
/// Properties editor for whichever object is active.
pub struct BActiveObjectRule;

impl BRule for BActiveObjectRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == CtxKey::Object.path()
    }
    fn after(&self, _: &BPath, name: &str, s: &mut BState) {
        crate::v2::scene::source::with_loaded(|loaded| {
            if let Some(model) = loaded {
                seed(s, model, name);
            }
        });
    }
}

/// Write `name`'s transform and first material into `ctx.buts.*`. Nothing
/// is written for an object the model does not have, so the panels keep
/// showing the last real values rather than zeros. The transform is the
/// object's own (`matrix_local`) — a child's is relative to its parent, as
/// Blender's panel shows it — for a camera, light or empty as for a mesh.
pub fn seed(state: &mut BState, model: &SceneModel, name: &str) {
    use crate::v2::scene::transform::{BTransform, BTransforms};
    let edit = crate::v2::scene::edit::BSceneEdit::now(state);
    if let Some(object) = model.objects.iter().find(|object| object.name == name) {
        set::seed_transform(state, object);
        if let Some(material) = object.materials.first() {
            set::seed_material(state, material);
        }
    }
    if let Some(local) = BTransforms::file_local(&edit, model, name) {
        set::seed_edited(state, &BTransform::of_matrix(local));
    }
    // What the panel set for it this session, over what the file says.
    if let Some(t) = crate::v2::scene::transform::BTransforms::now(state).get(name) {
        set::seed_edited(state, &t);
    }
}
