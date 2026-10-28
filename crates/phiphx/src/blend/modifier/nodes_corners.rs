//! Corner (split) normals of an evaluated mesh whose edges can be sharp —
//! Blender's `mesh_normals.cc`, which the Smooth by Angle result draws
//! with: `Mesh::corner_normals` picks the domain (`Mesh::normals_domain`),
//! `normals_calc_faces` gives every face its Newell normal
//! (`normal_calc_ngon`), `normals_calc_verts` the angle-weighted vertex
//! normals, and `normals_calc_corners` walks the fan of faces round each
//! vertex, stopping at a sharp edge, a sharp face, a non-manifold edge or a
//! winding flip, and gives every corner of a fan the fan's normal.
//!
//! The angle weights use Blender's `safe_acos_approx`, ported digit for
//! digit, so a normal here is the normal Blender draws rather than one a
//! few ULP off it.

use super::BlendEvalMesh;
use super::nodes_fans::fan_vertex;
use std::collections::HashSet;

pub(super) type V3 = [f32; 3];

pub(super) fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(super) fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// `math::normalize`: the zero vector stays zero.
pub(crate) fn normalize(v: V3) -> V3 {
    let l = dot(v, v).sqrt();
    if l == 0.0 { [0.0; 3] } else { [v[0] / l, v[1] / l, v[2] / l] }
}

/// `BLI_math_base.hh` `safe_acos_approx`: the polynomial Blender weighs
/// corner angles with (4.5e-5 max error against `acos`).
pub(crate) fn safe_acos_approx(x: f32) -> f32 {
    let f = x.abs();
    let m = if f < 1.0 { 1.0 - (1.0 - f) } else { 1.0 };
    let a = (1.0 - m).sqrt() * (1.5707963267 + m * (-0.213300989 + m * (0.077980478 + m * -0.02164095)));
    if x < 0.0 { std::f32::consts::PI - a } else { a }
}

/// `normal_calc_ngon`: Newell's method, `z = 1` for a degenerate face.
pub(crate) fn newell_normal(verts: &[V3], face: &[u32]) -> V3 {
    let mut n = [0.0f32; 3];
    let mut prev = verts[face[face.len() - 1] as usize];
    for &v in face {
        let cur = verts[v as usize];
        // `add_newell_cross_v3_v3v3`.
        n[0] += (prev[1] - cur[1]) * (prev[2] + cur[2]);
        n[1] += (prev[2] - cur[2]) * (prev[0] + cur[0]);
        n[2] += (prev[0] - cur[0]) * (prev[1] + cur[1]);
        prev = cur;
    }
    let n = normalize(n);
    if n == [0.0; 3] { [0.0, 0.0, 1.0] } else { n }
}

/// `bke::mesh::face_normal_calc`: a triangle's cross product, a quad's
/// diagonal cross product (`normal_quad_v3`), Newell for the rest.
pub(crate) fn face_normal_calc(verts: &[V3], face: &[u32]) -> V3 {
    let p = |i: usize| verts[face[i] as usize];
    let cross = |a: V3, b: V3| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let n = match face.len() {
        3 => normalize(cross(sub(p(0), p(1)), sub(p(1), p(2)))),
        4 => normalize(cross(sub(p(0), p(2)), sub(p(1), p(3)))),
        _ => return newell_normal(verts, face),
    };
    if n == [0.0; 3] { [0.0, 0.0, 1.0] } else { n }
}

/// Faces round each vertex, in face order (`vert_to_face_map`).
fn vert_faces(m: &BlendEvalMesh) -> Vec<Vec<usize>> {
    let mut out = vec![Vec::new(); m.vertices.len()];
    for (f, face) in m.faces.iter().enumerate() {
        for &v in face {
            if let Some(list) = out.get_mut(v as usize) {
                list.push(f);
            }
        }
    }
    out
}

/// `normals_calc_verts`: each face's normal weighted by its corner angle.
fn vertex_normals(m: &BlendEvalMesh, face_normals: &[V3], around: &[Vec<usize>]) -> Vec<V3> {
    (0..m.vertices.len())
        .map(|v| {
            let p = m.vertices[v];
            if around[v].is_empty() {
                return normalize(p);
            }
            let mut n = [0.0f32; 3];
            for &f in &around[v] {
                let face = m.faces.get(f);
                let Some(i) = face.iter().position(|&x| x == v as u32) else { continue };
                let (prev, next) = (face[(i + face.len() - 1) % face.len()], face[(i + 1) % face.len()]);
                let dp = normalize(sub(m.vertices[prev as usize], p));
                let dn = normalize(sub(m.vertices[next as usize], p));
                let w = safe_acos_approx(dot(dp, dn));
                for k in 0..3 {
                    n[k] += face_normals[f][k] * w;
                }
            }
            normalize(n)
        })
        .collect()
}

/// `Mesh::corner_normals` for `m`: one unit normal per face corner, in
/// face-corner order. Empty when the mesh has no faces.
pub(crate) fn corner_normals(m: &BlendEvalMesh) -> Vec<V3> {
    let corners = m.faces.corners();
    if corners == 0 {
        return Vec::new();
    }
    let in_range = |face: &[u32]| face.iter().all(|&v| (v as usize) < m.vertices.len());
    let face_normals: Vec<V3> =
        m.faces.iter().map(|f| if f.len() >= 3 && in_range(f) { newell_normal(&m.vertices, f) } else { [0.0, 0.0, 1.0] }).collect();
    let sharp_face = |f: usize| m.face_smooth.get(f).is_some_and(|s| !s);
    let sharp: HashSet<(u32, u32)> = m.sharp_edges.iter().map(|&(a, b)| (a.min(b), a.max(b))).collect();
    let around = vert_faces(m);
    // `Mesh::normals_domain`: nothing sharp anywhere is the point domain —
    // the vertex normals, gathered to the corners.
    let any_sharp_edge = sharp.len() > 0
        && m.faces.iter().any(|f| (0..f.len()).any(|i| {
            let (a, b) = (f[i], f[(i + 1) % f.len()]);
            sharp.contains(&(a.min(b), a.max(b)))
        }));
    if !any_sharp_edge && !(0..m.faces.len()).any(sharp_face) {
        let vn = vertex_normals(m, &face_normals, &around);
        return m.faces.iter().flatten().map(|&v| vn.get(v as usize).copied().unwrap_or([0.0, 0.0, 1.0])).collect();
    }
    let mut out = vec![[0.0f32; 3]; corners];
    let starts: Vec<usize> = m.faces.iter().scan(0, |s, f| { let at = *s; *s += f.len(); Some(at) }).collect();
    for v in 0..m.vertices.len() {
        if around[v].is_empty() {
            continue;
        }
        fan_vertex(m, v as u32, &around[v], &starts, &face_normals, &sharp, &sharp_face, &mut out);
    }
    out
}
