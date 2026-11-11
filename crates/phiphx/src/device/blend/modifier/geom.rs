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

/// Boundary edges: those used by exactly one face, in sorted order.
///
/// Every corner's edge into one list, sorted once, and the edges that occur
/// once kept — as `topology.rs` builds its edges. It used to insert each new
/// edge into a sorted list as it went, which moves half the list every time:
/// the crystal caves city's APRON-BASE, half a million faces once
/// subdivided, spent six minutes of the file's open in here.
pub(super) fn boundary(mesh: &BlendEvalMesh) -> Vec<(u32, u32)> {
    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(mesh.faces.corners());
    for face in 0..mesh.faces.len() {
        let corners = mesh.faces.get(face);
        for n in 0..corners.len() {
            let a = corners[n];
            let b = corners[(n + 1) % corners.len()];
            edges.push(if a < b { (a, b) } else { (b, a) });
        }
    }
    edges.sort_unstable();
    edges.chunk_by(|a, b| a == b).filter(|uses| uses.len() == 1).map(|uses| uses[0]).collect()
}

/// Open edges as `(face, corner, next corner)` in the owning face's winding:
/// the edges the Solidify's rim closes (`edge_users` in
/// `MOD_solidify_extrude.cc`).
pub(super) fn open_corners(mesh: &BlendEvalMesh) -> Vec<(usize, usize, usize)> {
    let open: std::collections::HashSet<(u32, u32)> = boundary(mesh).into_iter().collect();
    let mut out = Vec::new();
    for face in 0..mesh.faces.len() {
        let c = mesh.faces.get(face);
        for n in 0..c.len() {
            let (a, b) = (c[n], c[(n + 1) % c.len()]);
            if open.contains(&(a.min(b), a.max(b))) {
                out.push((face, n, (n + 1) % c.len()));
            }
        }
    }
    out
}
