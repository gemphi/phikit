//! The vertex normals the simple Solidify offsets along:
//! `mesh->vert_normals_true()` in `MOD_solidify_extrude.cc`, which is
//! `mesh::normals_calc_faces` then `mesh::normals_calc_verts`
//! (`mesh_normals.cc`) — unit face normals, each weighted at a vertex by the
//! corner's angle there.
//!
//! The area-weighted normals of `geom.rs` lean towards the bigger faces; on
//! a mesh of uneven quads (the CW-POSES `console_middle_geo`, Mirror →
//! Solidify) that put shell vertices up to 6 mm from Blender's.

use super::super::BlendEvalMesh;
use glam::Vec3;

/// `normalize_v3`: the zero vector stays zero.
fn normalize(v: Vec3) -> Vec3 {
    v.normalize_or_zero()
}

/// `math::safe_acos_approx` (`BLI_math_base.hh`), verbatim.
fn safe_acos_approx(x: f32) -> f32 {
    let f = x.abs();
    // Clamp and crush denormals.
    let m = match f < 1.0 {
        true => 1.0 - (1.0 - f),
        false => 1.0,
    };
    let a = (1.0 - m).sqrt() * (1.570_796_3 + m * (-0.213_300_99 + m * (0.077_980_48 + m * -0.021_640_95)));
    match x < 0.0 {
        true => std::f32::consts::PI - a,
        false => a,
    }
}

/// `normal_calc_ngon`, which `normals_calc_faces` uses for every face
/// whatever its size: Newell's sum (`add_newell_cross_v3_v3v3`) from the
/// last corner round; a degenerate face points up `+Z`.
fn face_normal(p: &[[f32; 3]], face: &[u32]) -> Vec3 {
    let at = |k: usize| Vec3::from(p[face[k] as usize]);
    let mut sum = Vec3::ZERO;
    let mut prev = at(face.len() - 1);
    for k in 0..face.len() {
        let curr = at(k);
        sum += Vec3::new(
            (prev.y - curr.y) * (prev.z + curr.z),
            (prev.z - curr.z) * (prev.x + curr.x),
            (prev.x - curr.x) * (prev.y + curr.y),
        );
        prev = curr;
    }
    let normal = normalize(sum);
    match normal == Vec3::ZERO {
        true => Vec3::Z,
        false => normal,
    }
}

/// `normals_calc_verts`: per vertex, each face's normal times the angle of
/// its corner there (`face_find_adjacent_verts`); a vertex no face uses takes
/// its own position, normalized.
pub(super) fn vertex_normals(mesh: &BlendEvalMesh) -> Vec<Vec3> {
    let p = &mesh.vertices;
    let mut out = vec![Vec3::ZERO; p.len()];
    let mut used = vec![false; p.len()];
    for face in mesh.faces.iter().filter(|f| f.len() >= 3) {
        let normal = face_normal(p, face);
        let n = face.len();
        for k in 0..n {
            let v = face[k] as usize;
            let here = Vec3::from(p[v]);
            let dir_prev = normalize(Vec3::from(p[face[(k + n - 1) % n] as usize]) - here);
            let dir_next = normalize(Vec3::from(p[face[(k + 1) % n] as usize]) - here);
            out[v] += normal * safe_acos_approx(dir_prev.dot(dir_next));
            used[v] = true;
        }
    }
    for (v, n) in out.iter_mut().enumerate() {
        *n = match used[v] {
            true => normalize(*n),
            false => normalize(Vec3::from(p[v])),
        };
    }
    out
}
