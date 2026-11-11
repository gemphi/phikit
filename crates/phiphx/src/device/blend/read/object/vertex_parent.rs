//! Vertex parenting — Blender `BKE_object_get_parent_matrix` for `PARVERT1`
//! and `PARVERT3` (`blenkernel/intern/object.cc`: `give_parvert`,
//! `ob_parvert3`) and `tri_to_quat` (`blenlib/intern/math_rotation_c.cc`).
//!
//! A child hung from one vertex (`PARVERT1`) takes the parent's world
//! matrix with its translation replaced by that vertex's world position.
//! A child hung from three (`PARVERT3`) takes the parent's world matrix
//! times a frame built on the triangle: its normal brought to Z and the
//! `v1 → v2` edge to X (`tri_to_quat`), standing at the triangle's
//! centroid. The vertices come from the parent's **evaluated** mesh —
//! `BKE_object_get_evaluated_mesh(par)` — by index, which is why a parent
//! under a Subdivision or an Armature places its children where the
//! modifier put the vertex, not where the file saved it. An index past the
//! evaluated mesh gives the parent's origin (`give_parvert` leaves the
//! vector zero when nothing was counted).
//!
//! The `CD_ORIGINDEX` branch of `give_parvert` — the average of every
//! evaluated vertex that came from the saved one — needs an original-index
//! layer the viewport's evaluation never adds (`mesh_calc_modifiers` adds
//! it only in weight or vertex paint, or for a modifier that asks), so the
//! index-into-evaluated rule is the one Blender takes for these files.

use glam::{Mat3, Mat4, Quat, Vec3};

/// `eObject_Partype`: a child on one of its parent's vertices.
pub const PARVERT1: u8 = 5;
/// `eObject_Partype`: a child on three of its parent's vertices.
pub const PARVERT3: u8 = 6;

/// Whether `partype` (masked by `PARTYPE`) is a vertex parent.
pub fn is_vertex_parent(partype: u8) -> bool {
    matches!(partype & 0x0F, PARVERT1 | PARVERT3)
}

/// `normal_tri_v3`: the unit normal of `v1 v2 v3` (zero when degenerate).
fn normal_tri(v1: Vec3, v2: Vec3, v3: Vec3) -> Vec3 {
    (v1 - v2).cross(v2 - v3).normalize_or_zero()
}

/// `tri_to_quat` — `tri_to_quat_ex` on the triangle's own normal: the
/// rotation that takes the Z axis onto the normal and the X axis onto the
/// `v1 → v2` edge projected into the triangle's plane.
pub(crate) fn tri_to_quat(v1: Vec3, v2: Vec3, v3: Vec3) -> Quat {
    let vec = normal_tri(v1, v2, v3);
    // "move z-axis to face-normal"
    let mut n = Vec3::new(vec.y, -vec.x, 0.0).normalize_or_zero();
    if n.x == 0.0 && n.y == 0.0 {
        n.x = 1.0;
    }
    let angle = -0.5 * vec.z.clamp(-1.0, 1.0).acos();
    let (si, co) = angle.sin_cos();
    let q1 = Quat::from_xyzw(n.x * si, n.y * si, 0.0, co);
    // "rotate back line v1-v2"
    let imat = Mat3::from_quat(q1).inverse();
    let mut edge = imat * (v2 - v1);
    edge.z = 0.0;
    let edge = edge.normalize_or_zero();
    // "what angle has this line with x-axis?"
    let angle = 0.5 * edge.y.atan2(edge.x);
    let (si, co) = angle.sin_cos();
    let q2 = Quat::from_xyzw(0.0, 0.0, si, co);
    q1 * q2
}

/// `give_parvert` on a mesh parent: vertex `nr` of its evaluated mesh, or
/// the origin when the index is past it.
fn parent_vertex(verts: &[[f32; 3]], nr: i32) -> Vec3 {
    usize::try_from(nr).ok().and_then(|i| verts.get(i)).map_or(Vec3::ZERO, |v| Vec3::from(*v))
}

/// `ob_parvert3`: the frame on the three parent vertices, in the parent's
/// local space.
fn parvert3(verts: &[[f32; 3]], par: [i32; 3]) -> Mat4 {
    let [v1, v2, v3] = par.map(|nr| parent_vertex(verts, nr));
    let mut m = Mat4::from_mat3(Mat3::from_quat(tri_to_quat(v1, v2, v3)));
    m.w_axis = ((v1 + v2 + v3) / 3.0).extend(1.0);
    m
}

/// `BKE_object_get_parent_matrix` for a vertex parent: what the child's
/// `parentinv · local` is composed onto. `parent_world` is the parent's
/// world matrix and `verts` its evaluated vertices in its own space.
/// `partype` is masked here; a non-vertex type returns `parent_world`.
pub(crate) fn parent_matrix(partype: u8, par: [i32; 3], parent_world: Mat4, verts: &[[f32; 3]]) -> Mat4 {
    match partype & 0x0F {
        PARVERT1 => Mat4::from_translation(parent_world.transform_point3(parent_vertex(verts, par[0]))),
        PARVERT3 => parent_world * parvert3(verts, par),
        _ => parent_world,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Mat4, b: Mat4) -> bool {
        a.to_cols_array().iter().zip(b.to_cols_array()).all(|(x, y)| (x - y).abs() < 1e-5)
    }

    #[test]
    fn one_vertex_takes_only_the_place() {
        let world = Mat4::from_rotation_z(0.7) * Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let verts = [[0.0, 0.0, 0.0], [1.0, 0.5, -0.25]];
        let m = parent_matrix(PARVERT1, [1, 0, 0], world, &verts);
        assert!(close(m, Mat4::from_translation(world.transform_point3(Vec3::new(1.0, 0.5, -0.25)))));
    }

    #[test]
    fn a_flat_triangle_on_the_xy_plane_is_the_identity_frame() {
        // Normal +Z, v1 → v2 along +X: Blender's `tri_to_quat` is the unit
        // quaternion here, so the frame is a translation to the centroid.
        let verts = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let m = parent_matrix(PARVERT3, [0, 1, 2], Mat4::IDENTITY, &verts);
        assert!(close(m, Mat4::from_translation(Vec3::new(2.0 / 3.0, 1.0, 0.0))), "{m}");
    }

    #[test]
    fn the_frame_carries_the_normal_to_z_and_the_edge_to_x() {
        let (v1, v2, v3) = (Vec3::new(0.3, -0.2, 1.0), Vec3::new(1.4, 0.1, 0.6), Vec3::new(0.5, 1.3, 1.9));
        let rot = Mat3::from_quat(tri_to_quat(v1, v2, v3));
        let n = normal_tri(v1, v2, v3);
        assert!((rot * Vec3::Z - n).length() < 1e-5, "Z lands on the normal");
        let edge = (v2 - v1).normalize();
        assert!((rot * Vec3::X).dot(edge) > 0.999, "X lands along v1 → v2");
    }

    #[test]
    fn an_index_past_the_mesh_is_the_origin() {
        let verts = [[1.0, 1.0, 1.0]];
        let m = parent_matrix(PARVERT1, [7, 0, 0], Mat4::IDENTITY, &verts);
        assert!(close(m, Mat4::IDENTITY));
    }
}
