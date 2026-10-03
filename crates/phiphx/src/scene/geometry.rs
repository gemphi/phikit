//! Pure geometry helpers shared by the blend and phx mappers: per-vertex
//! UVs from per-corner UVs, and normal selection / synthesis.

use super::SceneNormals;

/// Build a per-vertex UV buffer from the object's per-loop UVs.
///
/// Blender stores UVs per loop (corner), so a vertex shared by multiple faces
/// can carry several UVs. For flat-colour rendering we only need a stable
/// per-vertex sample, so each vertex takes the UV of the first loop that
/// references it. Returns an empty vec when the source carried no UV layer,
/// which the renderer reads as "no UVs".
pub(super) fn per_vertex_uvs(faces: &crate::blend::BFaces, loop_uvs: &[[f32; 2]], vert_count: usize) -> Vec<[f32; 2]> {
    if loop_uvs.is_empty() || vert_count == 0 {
        return Vec::new();
    }
    let mut uvs = vec![[0.0f32, 0.0]; vert_count];
    let mut seen = vec![false; vert_count];
    let mut loop_index = 0usize;
    for face in faces.iter() {
        for &vert in face {
            let v = vert as usize;
            if v < vert_count && !seen[v] {
                if let Some(&uv) = loop_uvs.get(loop_index) {
                    uvs[v] = uv;
                    seen[v] = true;
                }
            }
            loop_index += 1;
        }
    }
    uvs
}

/// Build a per-face normal set for the triangulated mesh.
///
/// When `loop_normals` covers a triangle's leading corner it is reused
/// (per-face), otherwise the normal is synthesized from the triangle's
/// winding (`(b - a) × (c - a)`, normalized). The result always has exactly
/// `tri_count` entries so it satisfies the [`SceneNormals::PerFace`]
/// invariant.
pub(super) fn build_normals(
    vertices: &[[f32; 3]],
    indices: &[u32],
    loop_normals: &[[f32; 3]],
    tri_loop_starts: &[usize],
    tri_count: usize,
    tri_smooth: &[bool],
) -> SceneNormals {
    // Custom split normals win (Blender draws them verbatim).
    if !loop_normals.is_empty() {
        let normals = (0..tri_count)
            .map(|t| {
                tri_loop_starts
                    .get(t)
                    .and_then(|&start| loop_normals.get(start))
                    .copied()
                    .unwrap_or_else(|| tri_face_normal(vertices, indices, t))
            })
            .collect();
        return SceneNormals::PerFace(normals);
    }
    let face: Vec<[f32; 3]> = (0..tri_count).map(|t| tri_face_normal(vertices, indices, t)).collect();
    if !tri_smooth.iter().any(|s| *s) {
        return SceneNormals::PerFace(face);
    }
    // Smooth faces: corner-angle-weighted vertex normals over the smooth
    // triangles (Blender `mesh::normals_calc_verts`), then each smooth
    // triangle takes the mean of its three corner normals — the flat-fill
    // painter's closest equivalent of Gouraud shading.
    let accum = smooth_vertex_normals(vertices, indices, &face, tri_smooth);
    let normals = (0..tri_count)
        .map(|t| {
            if !tri_smooth.get(t).copied().unwrap_or(false) {
                return face[t];
            }
            let mut n = [0.0f32; 3];
            for k in 0..3 {
                let v = accum[indices[t * 3 + k] as usize];
                for a in 0..3 {
                    n[a] += v[a];
                }
            }
            normalize(n).unwrap_or(face[t])
        })
        .collect();
    SceneNormals::PerFace(normals)
}

pub(super) fn tri_face_normal(vertices: &[[f32; 3]], indices: &[u32], t: usize) -> [f32; 3] {
    let base = t * 3;
    face_normal(
        vertices[indices[base] as usize],
        vertices[indices[base + 1] as usize],
        vertices[indices[base + 2] as usize],
    )
}

/// Corner-angle-weighted vertex normals over the smooth triangles
/// (Blender `mesh::normals_calc_verts`), unnormalised.
pub(super) fn smooth_vertex_normals(vertices: &[[f32; 3]], indices: &[u32], face: &[[f32; 3]], smooth: &[bool]) -> Vec<[f32; 3]> {
    let mut accum = vec![[0.0f32; 3]; vertices.len()];
    for t in (0..face.len()).filter(|&t| smooth.get(t).copied().unwrap_or(false)) {
        let ids = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]].map(|i| i as usize);
        for k in 0..3 {
            let w = corner_angle(vertices[ids[k]], vertices[ids[(k + 1) % 3]], vertices[ids[(k + 2) % 3]]);
            for a in 0..3 {
                accum[ids[k]][a] += face[t][a] * w;
            }
        }
    }
    accum
}

/// Interior angle at `p` of the triangle `(p, a, b)`.
fn corner_angle(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let u = normalize([a[0] - p[0], a[1] - p[1], a[2] - p[2]]);
    let v = normalize([b[0] - p[0], b[1] - p[1], b[2] - p[2]]);
    match (u, v) {
        (Some(u), Some(v)) => (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]).clamp(-1.0, 1.0).acos(),
        _ => 0.0,
    }
}

pub(super) fn normalize(n: [f32; 3]) -> Option<[f32; 3]> {
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    (len > f32::EPSILON).then(|| [n[0] / len, n[1] / len, n[2] / len])
}

/// Normalized normal of the triangle `(a, b, c)` via `(b - a) × (c - a)`.
///
/// Falls back to `[0, 0, 1]` for a degenerate (zero-area) triangle so the
/// result is always a finite unit-ish vector.
pub(super) fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > f32::EPSILON {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}
