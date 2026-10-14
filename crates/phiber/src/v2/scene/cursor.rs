//! The 3D cursor: where new objects go and what the Snap menu snaps to
//! (`View3DCursor`).
//!
//! The file's cursor is read with it (phiphx `BlendCursor`); the session's
//! is one key over it (`ctx.scene.cursor`, `x,y,z,w,qx,qy,qz` — its place
//! and its turn), so Undo puts it back and a save writes it
//! (`scene::save`). It moves by:
//! - Shift + right-click on the canvas (`view3d.cursor3d` with Blender's
//!   defaults, `use_depth` and `orientation='VIEW'`): onto the surface under
//!   the pointer — or, over empty space, onto the plane through the cursor
//!   facing the view — turned to the view (`hit.rs`);
//! - Object ▸ Snap: Cursor to World Origin, to Selected (the selection's
//!   median point, `snap_curs_to_sel_ex`), to Active; and Selection to
//!   Cursor, which moves each selected object's origin onto it;
//! - View ▸ Align View ▸ Center Cursor and Frame All (Shift C).

use crate::phiphx::scene::SceneModel;
use crate::v2::scene::BObject3D;
use crate::v2::scene::transform::{BTransform, BTransforms};
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, SceneKey, log, rule::BRule};
use glam::{Mat3, Mat4, Quat, Vec3};
use std::collections::HashMap;

/// The 3D cursor: its place and its turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BCursor {
    pub location: Vec3,
    pub rotation: Quat,
}

impl Default for BCursor {
    fn default() -> Self {
        Self { location: Vec3::ZERO, rotation: Quat::IDENTITY }
    }
}

impl BCursor {
    /// Where it is now: as the session left it, else as the file has it.
    pub fn now(s: &BState) -> Self {
        let file = || crate::v2::scene::source::with_loaded(|m| m.map(|m| Self::of_matrix(Mat4::from_cols_array_2d(&m.cursor.matrix))));
        s.get(SceneKey::Cursor).and_then(Self::parse).or_else(file).unwrap_or_default()
    }

    /// The session's cursor, when it moved one — what a save writes.
    pub fn edited(s: &BState) -> Option<[[f32; 4]; 4]> {
        s.get(SceneKey::Cursor).and_then(Self::parse).map(|c| c.matrix().to_cols_array_2d())
    }

    pub fn matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.location)
    }

    fn of_matrix(m: Mat4) -> Self {
        let (_, rotation, location) = m.to_scale_rotation_translation();
        Self { location, rotation }
    }

    fn parse(raw: &str) -> Option<Self> {
        let v: Vec<f32> = raw.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        (v.len() == 7).then(|| Self { location: Vec3::new(v[0], v[1], v[2]), rotation: Quat::from_xyzw(v[4], v[5], v[6], v[3]).normalize() })
    }

    /// As `ctx.scene.cursor` holds it.
    pub fn encode(&self) -> String {
        let (l, q) = (self.location, self.rotation);
        [l.x, l.y, l.z, q.w, q.x, q.y, q.z].map(|v| v.to_string()).join(",")
    }

    pub fn write(&self, s: &mut BState) {
        s.set(SceneKey::Cursor, self.encode());
    }

    /// Placed by a click at the canvas point `(su, sv)` of `view`
    /// (`view3d.cursor3d`): on the nearest surface of `scene` under it, or
    /// on the plane through the cursor now that faces the view — turned to
    /// the view. `None` when the point looks along that plane.
    pub fn under(&self, scene: &[BObject3D], view: &crate::v2::scene::pick::BPickView, (su, sv): (f32, f32)) -> Option<Self> {
        let (ray, (right, up, back)) = (view.ray(su, sv), view.axes());
        let on_plane = || {
            let facing = ray.dir.dot(-back);
            (facing.abs() > 1e-6).then(|| ray.origin + ray.dir * ((self.location - ray.origin).dot(-back) / facing))
        };
        let at = crate::v2::scene::pick::pick(scene, ray).map(|hit| ray.origin + ray.dir * hit.distance).or_else(on_plane)?;
        Some(Self { location: at, rotation: Quat::from_mat3(&Mat3::from_cols(right, up, back)).normalize() })
    }
}

/// Every object's world now, by its name now: the meshes and rigs as drawn,
/// the cameras, lights and empties where the edits put them.
fn worlds(s: &BState, model: Option<&SceneModel>) -> HashMap<String, Mat4> {
    let mut out = HashMap::new();
    if let Some(model) = model {
        let before = BTransforms::posed(s, model, crate::v2::scene::motion::shown_frame(s));
        let edits = BTransforms::now(s);
        out = if edits.list.is_empty() { before } else { edits.worlds(s, model, &before) };
    }
    for object in crate::v2::scene::source::scene_objects(s).iter() {
        let m = match object {
            BObject3D::Primitive { name, x, y, .. } => (name, Mat4::from_translation(Vec3::new(x / crate::v2::scene::MESH_SCALE, y / crate::v2::scene::MESH_SCALE, 0.0))),
            BObject3D::Loaded { name, matrix_world, .. } => (name, Mat4::from_cols_array_2d(matrix_world)),
        };
        out.insert(m.0.clone(), m.1);
    }
    out
}

/// A point rounded to the nearest world unit — Blender's default grid.
fn grid(p: Vec3) -> Vec3 {
    p.round()
}

/// `ctx.op.cursor` (see the module docs).
pub struct BCursorRule;

impl BRule for BCursorRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Cursor.path()
    }
    fn after(&self, _: &BPath, op: &str, s: &mut BState) {
        let model = crate::v2::scene::source::loaded_shared();
        let worlds = worlds(s, model.as_deref());
        let place = |name: &str| worlds.get(name).map(|m| m.w_axis.truncate());
        let selection = crate::v2::scene::select::BSelection::now(s);
        let cursor = BCursor::now(s);
        let moved = |location: Vec3| BCursor { location, ..cursor };
        let to = match op {
            "origin" => Some(moved(Vec3::ZERO)),
            "center_frame_all" => {
                moved(Vec3::ZERO).write(s);
                s.set(OpKey::View3d, "frame_all");
                crate::v2::scene::view_op::BView3dOpRule.after(&OpKey::View3d.path(), "frame_all", s);
                log::push("Centered the cursor and framed all".to_string());
                return;
            }
            "active" => s.get(CtxKey::Object).and_then(place).map(moved),
            "selected" => {
                let points: Vec<Vec3> = selection.list.iter().filter_map(|n| place(n)).collect();
                (!points.is_empty()).then(|| moved(points.iter().sum::<Vec3>() / points.len() as f32))
            }
            // `view3d.snap_cursor_to_grid`: the cursor rounded to the nearest
            // world unit (Blender's default grid scale is 1).
            "cursor_to_grid" => Some(moved(grid(cursor.location))),
            // `view3d.snap_selected_to_grid` / `_to_active`: each selected
            // object's origin to the nearest grid point, or onto the active
            // object. The transform machinery is `selection_to_cursor`'s —
            // only the per-object target differs — so it is one closure.
            "selection_to_cursor" | "selection_to_grid" | "selection_to_active" => {
                let Some(model) = model.as_deref() else { return };
                let active_at = s.get(CtxKey::Object).and_then(|n| place(n));
                if op == "selection_to_active" && active_at.is_none() {
                    return;
                }
                let target_for = |world: &Mat4| -> Vec3 {
                    match op {
                        "selection_to_grid" => grid(world.w_axis.truncate()),
                        "selection_to_active" => active_at.unwrap_or(cursor.location),
                        _ => cursor.location,
                    }
                };
                let mut edits = BTransforms::now(s);
                let edit = crate::v2::scene::edit::BSceneEdit::now(s);
                for name in &selection.list {
                    let Some(world) = worlds.get(name) else { continue };
                    // The new world is the old one with its place at the
                    // target; the object's own part takes the difference:
                    // `local · world⁻¹ · world′`.
                    let local = edits.get(name).map(|t| t.matrix()).or_else(|| BTransforms::file_local(&edit, model, name)).unwrap_or(*world);
                    let mut target = *world;
                    target.w_axis = target_for(world).extend(1.0);
                    edits.set(name, BTransform::of_matrix(local * world.inverse() * target));
                }
                edits.write(s);
                let where_to = match op { "selection_to_grid" => "the grid", "selection_to_active" => "the active object", _ => "the cursor" };
                log::push(format!("Moved {} object(s) to {where_to}", selection.list.len()));
                return;
            }
            _ => None,
        };
        if let Some(to) = to {
            to.write(s);
            log::push("Moved the 3D cursor".to_string());
        }
    }
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod tests;
