//! Where every object stands once the edits are in — meshes, cameras,
//! lights and empties alike, so a light parented to a cube follows it and
//! G on a camera moves the camera. An edited object's world is its old one
//! with its own part replaced (`world · local⁻¹ · local′`), and every
//! child's moves by what its parent's did — `BKE_object_where_is_calc`
//! placing them from the new values.

use super::BTransforms;
use crate::phiphx::scene::SceneModel;
use crate::v2::scene::BObject3D;
use crate::v2::scene::edit::BSceneEdit;
use crate::v2::scene::links::BSceneTree;
use crate::v2::state::BState;
use glam::Mat4;
use std::collections::HashMap;

/// What working out one world needs.
struct BWorldWalk<'a> {
    edits: &'a BTransforms,
    old: &'a HashMap<String, Mat4>,
    tree: &'a BSceneTree,
    edit: &'a BSceneEdit,
    model: &'a SceneModel,
}

impl BWorldWalk<'_> {
    fn world(&self, name: &str, depth: u32, memo: &mut HashMap<String, Mat4>) -> Mat4 {
        if let Some(m) = memo.get(name) {
            return *m;
        }
        let before = self.old[name];
        let own = match self.edits.get(name) {
            Some(t) => before * BTransforms::file_local(self.edit, self.model, name).unwrap_or(before).inverse() * t.matrix(),
            None => before,
        };
        let after = match self.tree.parent_of(name).filter(|p| self.old.contains_key(*p) && depth < 64) {
            Some(parent) => self.world(parent, depth + 1, memo) * self.old[parent].inverse() * own,
            None => own,
        };
        memo.insert(name.to_string(), after);
        after
    }
}

impl BTransforms {
    /// The object's own matrix as the file has it (`Object.matrix_local`),
    /// through the session's renames; `None` for an object the session made.
    pub fn file_local(edit: &BSceneEdit, model: &SceneModel, name: &str) -> Option<Mat4> {
        let file = edit.renamed.iter().find(|(new, _)| new == name).map_or(name, |(_, old)| old.as_str());
        match model.objects.iter().position(|o| o.name == file) {
            Some(at) => model.locals.get(at).map(Mat4::from_cols_array_2d),
            None => model.extra_locals.iter().find(|(n, _)| n == file).map(|(_, m)| Mat4::from_cols_array_2d(m)),
        }
    }

    /// Every file object's world before the edits at `frame`, by its name
    /// now — meshes, cameras, lights and empties, where the animation puts
    /// them while the scene moves.
    pub fn posed(s: &BState, model: &SceneModel, frame: Option<f32>) -> HashMap<String, Mat4> {
        let edit = BSceneEdit::now(s);
        let now = |file: &str| edit.renamed.iter().find(|(_, old)| old == file).map_or(file.to_string(), |(new, _)| new.clone());
        let posed = crate::v2::scene::motion::posed(model, frame);
        let p = posed.as_ref();
        let mut out = HashMap::new();
        for (i, o) in model.objects.iter().enumerate() {
            out.insert(now(&o.name), p.and_then(|p| p.object(i)).unwrap_or(o.matrix_world));
        }
        for (i, c) in model.cameras.iter().enumerate() {
            out.insert(now(&c.name), p.and_then(|p| p.camera(i)).unwrap_or(c.matrix_world));
        }
        for (i, l) in model.lights.iter().enumerate() {
            out.insert(now(&l.name), p.and_then(|p| p.light(i)).unwrap_or(l.matrix_world));
        }
        for (i, e) in model.empties.iter().enumerate() {
            out.insert(now(&e.name), p.and_then(|p| p.empty(i)).unwrap_or(e.matrix_world));
        }
        out.into_iter().map(|(name, m)| (name, Mat4::from_cols_array_2d(&m))).collect()
    }

    /// `old` with the edits in.
    pub fn worlds(&self, s: &BState, model: &SceneModel, old: &HashMap<String, Mat4>) -> HashMap<String, Mat4> {
        let (edit, tree) = (BSceneEdit::now(s), BSceneTree::now(s, model));
        let walk = BWorldWalk { edits: self, old, tree: &tree, edit: &edit, model };
        let mut memo = HashMap::new();
        for name in old.keys() {
            walk.world(name, 0, &mut memo);
        }
        memo
    }

    /// Move the edited meshes of `objects`, their children with them —
    /// a mesh parented to an edited camera, light or empty too.
    pub fn apply(&self, s: &BState, model: &SceneModel, objects: &mut [BObject3D]) {
        if self.list.is_empty() {
            return;
        }
        let mut old = Self::posed(s, model, crate::v2::scene::motion::shown_frame(s));
        for object in objects.iter() {
            if let BObject3D::Loaded { name, matrix_world, .. } = object {
                old.insert(name.clone(), Mat4::from_cols_array_2d(matrix_world));
            }
        }
        let after = self.worlds(s, model, &old);
        for object in objects.iter_mut() {
            if let BObject3D::Loaded { name, matrix_world, location, scale, .. } = object {
                *matrix_world = after[name.as_str()].to_cols_array_2d();
                if let Some(t) = self.get(name) {
                    (*location, *scale) = (t.location, t.scale);
                }
            }
        }
    }

    /// The cameras', lights' and empties' worlds after the edits at
    /// `frame`, by their names in the file; `None` while nothing is edited.
    pub fn extra_worlds(&self, s: &BState, model: &SceneModel, frame: Option<f32>) -> Option<HashMap<String, Mat4>> {
        if self.list.is_empty() {
            return None;
        }
        let edit = BSceneEdit::now(s);
        let after = self.worlds(s, model, &Self::posed(s, model, frame));
        let now = |file: &str| edit.renamed.iter().find(|(_, old)| old == file).map_or(file.to_string(), |(new, _)| new.clone());
        let files = model.cameras.iter().map(|c| &c.name).chain(model.lights.iter().map(|l| &l.name)).chain(model.empties.iter().map(|e| &e.name));
        Some(files.filter_map(|file| after.get(&now(file)).map(|m| (file.clone(), *m))).collect())
    }
}
