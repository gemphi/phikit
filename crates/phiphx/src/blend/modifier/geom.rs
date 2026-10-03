//! Mesh geometry shared by several modifiers: vertex normals and the open
//! boundary. Blender computes both once per evaluated mesh; here each
//! modifier that needs them asks for them, which keeps the stack order free.

use super::BlendEvalMesh;
use glam::Vec3;

/// Area-weighted vertex normals, as Blender's deform modifiers use.
pub(super) fn vertex_normals(mesh: &BlendEvalMesh) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; mesh.vertices.len()];
    for face in 0..mesh.faces.len() {
        let corners = mesh.faces.get(face);
        if corners.len() < 3 {
            continue;
        }
        let at = |n: usize| Vec3::from(mesh.vertices[corners[n] as usize]);
        // Newell's normal, whose length is twice the face's area — which is
        // the weight Blender wants.
        let mut normal = Vec3::ZERO;
        for n in 0..corners.len() {
            normal += at(n).cross(at((n + 1) % corners.len()));
        }
        for corner in corners {
            normals[*corner as usize] += normal;
        }
    }
    for normal in &mut normals {
        *normal = normal.normalize_or_zero();
    }
    normals
}

/// Boundary edges: those used by exactly one face.
pub(super) fn boundary(mesh: &BlendEvalMesh) -> Vec<(u32, u32)> {
    let mut edges: Vec<((u32, u32), u32)> = Vec::new();
    for face in 0..mesh.faces.len() {
        let corners = mesh.faces.get(face);
        for n in 0..corners.len() {
            let a = corners[n];
            let b = corners[(n + 1) % corners.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            match edges.binary_search_by_key(&key, |(k, _)| *k) {
                Ok(at) => edges[at].1 += 1,
                Err(at) => edges.insert(at, (key, 1)),
            }
        }
    }
    edges.into_iter().filter(|(_, uses)| *uses == 1).map(|(e, _)| e).collect()
}
