//! The Transform panel's edits — `Object.location`, `rotation_euler` and
//! `scale` typed into the Object tab: the object moves in the viewport at
//! once and a save writes it. One key over the file
//! (`ctx.scene.transformed`, `name=loc,rot,scale|…`, rotation in degrees
//! as the panel shows it), stepped by Undo like the other scene edits.
//!
//! An edited object's world matrix is its old one with its own part
//! replaced — `world · local⁻¹ · local′`, the parent's part kept — and its
//! children follow it (`world(child) · (world′(parent) · world(parent)⁻¹)`),
//! as `BKE_object_where_is_calc` would place them from the new values.

mod world;

use super::edit::BSceneEdit;
use crate::v2::state::{BKey as _, BPath, BState, ButsKey, CtxKey, OpKey, SceneKey, rule::BRule, val::{BValue, BValueKind}};
use glam::{EulerRot, Mat4, Quat, Vec3};

/// The panel's three vectors, and the keys it draws them from.
pub const FIELDS: [&str; 3] = ["location", "rotation_euler", "scale"];
const AXES: [&str; 3] = ["x", "y", "z"];

/// One object's transform: location, XYZ Euler rotation in degrees, scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BTransform {
    pub location: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl BTransform {
    /// No move, no turn, unit scale — what a cleared object has.
    pub const IDENTITY: Self = Self { location: [0.0; 3], rotation: [0.0; 3], scale: [1.0; 3] };

    /// `T · R · S`, R as Blender's XYZ Euler (`eul_to_mat3`: X first).
    pub fn matrix(&self) -> Mat4 {
        let [rx, ry, rz] = self.rotation.map(f32::to_radians);
        Mat4::from_scale_rotation_translation(Vec3::from(self.scale), Quat::from_euler(EulerRot::ZYX, rz, ry, rx), Vec3::from(self.location))
    }

    /// A matrix's own location, XYZ Euler rotation (degrees) and scale —
    /// `BKE_object_apply_mat4` without a parent.
    pub fn of_matrix(m: Mat4) -> Self {
        let (scale, rotation, location) = m.to_scale_rotation_translation();
        let (rz, ry, rx) = rotation.to_euler(EulerRot::ZYX);
        Self { location: location.into(), rotation: [rx, ry, rz].map(f32::to_degrees), scale: scale.into() }
    }

    /// As a save writes it: location, rotation in radians, scale.
    pub fn to_blend(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        (self.location, self.rotation.map(f32::to_radians), self.scale)
    }

    pub(crate) fn encode(&self) -> String {
        self.location.iter().chain(&self.rotation).chain(&self.scale).map(|v| format!("{v}")).collect::<Vec<_>>().join(",")
    }

    pub(crate) fn parse(raw: &str) -> Option<Self> {
        let v: Vec<f32> = raw.split(',').filter_map(|p| p.parse().ok()).collect();
        (v.len() == 9).then(|| Self { location: [v[0], v[1], v[2]], rotation: [v[3], v[4], v[5]], scale: [v[6], v[7], v[8]] })
    }

    /// What the panel's nine fields hold now.
    pub fn from_panel(s: &BState) -> Option<Self> {
        let read = |field: &'static str| -> Option<[f32; 3]> {
            let mut out = [0.0; 3];
            for (slot, axis) in out.iter_mut().zip(AXES) {
                *slot = match s.value(&ButsKey::Axis(field, axis).path(), BValueKind::Float) {
                    Some(BValue::Float(v)) => v,
                    _ => return None,
                };
            }
            Some(out)
        };
        Some(Self { location: read(FIELDS[0])?, rotation: read(FIELDS[1])?, scale: read(FIELDS[2])? })
    }
}

/// Every object the panel changed, by its name now.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BTransforms {
    pub list: Vec<(String, BTransform)>,
}

impl BTransforms {
    pub fn now(s: &BState) -> Self {
        Self::parse(s.get(SceneKey::Transformed).unwrap_or(""))
    }

    /// The list as `ctx.scene.transformed` spells it.
    pub fn parse(raw: &str) -> Self {
        let list = raw
            .split('|')
            .filter_map(|e| e.split_once('=').and_then(|(n, t)| BTransform::parse(t).map(|t| (n.to_string(), t))))
            .collect();
        Self { list }
    }

    pub fn encode(&self) -> String {
        self.list.iter().map(|(n, t)| format!("{n}={}", t.encode())).collect::<Vec<_>>().join("|")
    }

    pub fn write(&self, s: &mut BState) {
        s.set(SceneKey::Transformed, self.encode());
    }

    pub fn get(&self, name: &str) -> Option<BTransform> {
        self.list.iter().find(|(n, _)| n == name).map(|(_, t)| *t)
    }

    pub fn set(&mut self, name: &str, t: BTransform) {
        self.list.retain(|(n, _)| n != name);
        self.list.push((name.to_string(), t));
    }

    pub fn rename(&mut self, from: &str, to: &str) {
        self.list.iter_mut().filter(|(n, _)| n == from).for_each(|(n, _)| *n = to.to_string());
    }

    pub fn forget(&mut self, name: &str) {
        self.list.retain(|(n, _)| n != name);
    }

    /// The edits a save writes for objects the session made (added or
    /// duplicated — the file's own carry theirs in their object edit).
    pub fn session_edits(&self, edit: &BSceneEdit) -> Vec<crate::phiphx::blend::write::BlendObjectEdit> {
        let made = |name: &str| edit.added.iter().any(|(n, _)| n == name) || edit.copied.iter().any(|(n, _)| n == name);
        self.list
            .iter()
            .filter(|(name, _)| made(name))
            .map(|(name, t)| crate::phiphx::blend::write::BlendObjectEdit { name: name.clone(), transform: Some(t.to_blend()), ..Default::default() })
            .collect()
    }

    /// One number for the object list's memo.
    pub fn fingerprint(s: &BState) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        s.get(SceneKey::Transformed).hash(&mut h);
        s.get(SceneKey::Parented).hash(&mut h);
        h.finish()
    }
}

/// A Transform field typed in: the active object takes the panel's values.
pub struct BTransformRule;

impl BRule for BTransformRule {
    fn wants(&self, path: &BPath) -> bool {
        matches!(path.segs.as_ref(), ["buts", field, "x" | "y" | "z"] if FIELDS.contains(field))
    }
    fn after(&self, _: &BPath, _: &str, s: &mut BState) {
        let (Some(active), Some(t)) = (s.get(CtxKey::Object).map(str::to_string), BTransform::from_panel(s)) else { return };
        let mut all = BTransforms::now(s);
        all.set(&active, t);
        all.write(s);
    }
}

/// `object.location_clear` / `rotation_clear` / `scale_clear` — Object ▸ Clear
/// (Alt G / R / S): each selected object's channel goes back to identity, its
/// other channels kept. Blender clears per axis unless that axis is locked
/// (`object_transform.cc`'s `object_clear_loc` / `_rot` / `_scale`); phiber has
/// no transform locks yet, so it clears the whole channel — which is exactly
/// Blender's result for an unlocked object (the default), a recorded divergence
/// only for the locked case.
pub struct BObjectClearRule;

impl BRule for BObjectClearRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::ObjectClear.path()
    }

    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        s.remove(OpKey::ObjectClear);
        let Some(model) = crate::v2::scene::source::loaded_shared() else { return };
        let edit = BSceneEdit::now(s);
        let mut all = BTransforms::now(s);
        for name in &model.selection.selected {
            // Where the object stands now: its overlay transform, else the one
            // the file gives it, else identity.
            let current = all
                .get(name)
                .or_else(|| BTransforms::file_local(&edit, &model, name).map(BTransform::of_matrix))
                .unwrap_or(BTransform::IDENTITY);
            let next = match value {
                "location" => BTransform { location: [0.0; 3], ..current },
                "rotation" => BTransform { rotation: [0.0; 3], ..current },
                "scale" => BTransform { scale: [1.0; 3], ..current },
                _ => return,
            };
            all.set(name, next);
        }
        all.write(s);
        // The panel shows what the active object now has.
        if let Some(active) = s.get(CtxKey::Object).map(str::to_string) {
            if let Some(t) = BTransforms::now(s).get(&active) {
                crate::v2::part::props::active::set::seed_edited(s, &t);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_values_make_blenders_matrix() {
        let t = BTransform { location: [1.0, 2.0, 3.0], rotation: [0.0, 0.0, 90.0], scale: [2.0, 2.0, 2.0] };
        let m = t.matrix();
        // 90° about Z doubled: X lands on +2Y; then moved.
        assert!((m.transform_vector3(Vec3::X) - Vec3::new(0.0, 2.0, 0.0)).length() < 1e-5);
        assert!((m.w_axis.truncate() - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-6);
        // XYZ order: X first, then Y, then Z.
        let xyz = BTransform { location: [0.0; 3], rotation: [90.0, 0.0, 90.0], scale: [1.0; 3] }.matrix();
        assert!((xyz.transform_vector3(Vec3::Y) - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-5, "Y → Z by X, stays Z under Z");
        let mut s = BState::new();
        BTransforms { list: vec![("Cube".into(), t)] }.write(&mut s);
        assert_eq!(BTransforms::now(&s).get("Cube"), Some(t));
    }

    #[test]
    fn clear_sends_a_channel_back_to_identity_keeping_the_others() {
        use crate::phiphx::scene::{SceneCorners, SceneMesh, SceneModel, SceneNormals, SceneObject};
        use crate::v2::state::action::apply_action;
        use crate::v2::view::prim::BAction;

        let mesh = SceneMesh {
            vertices: vec![[0.0; 3]],
            indices: Vec::new(),
            normals: SceneNormals::PerFace(Vec::new()),
            corners: SceneCorners::default(),
        };
        let mut model = SceneModel { objects: vec![SceneObject::identity("Cube", mesh)], ..Default::default() };
        model.selection.selected = vec!["Cube".into()];
        crate::v2::scene::source::set_loaded_scene(Some(model));

        let mut s = BState::new();
        let posed = BTransform { location: [5.0, 1.0, -2.0], rotation: [0.0, 0.0, 90.0], scale: [2.0, 2.0, 2.0] };
        BTransforms { list: vec![("Cube".into(), posed)] }.write(&mut s);

        let clear = |s: &mut BState, which: &str| apply_action(s, &BAction::Set(OpKey::ObjectClear.path(), which.to_string()));

        clear(&mut s, "location");
        let after = BTransforms::now(&s).get("Cube").unwrap();
        assert_eq!(after.location, [0.0; 3], "location cleared");
        assert_eq!((after.rotation, after.scale), (posed.rotation, posed.scale), "rotation and scale kept");

        clear(&mut s, "rotation");
        assert_eq!(BTransforms::now(&s).get("Cube").unwrap().rotation, [0.0; 3], "rotation cleared");

        clear(&mut s, "scale");
        assert_eq!(BTransforms::now(&s).get("Cube").unwrap().scale, [1.0; 3], "scale back to one");

        crate::v2::scene::source::clear_loaded_scene();
    }
}
