//! The projections: not solvers at all, which is why they are instant and
//! why they distort where the surface turns away from the axis.
//!
//! Blender's `uv.cube_project`, `uv.cylinder_project`, `uv.sphere_project`
//! and `uv.project_from_view`. Each is the same shape of thing — a point
//! in space mapped to a point in the square — and the only question is
//! which point.

use crate::uv::{BUnwrap, BUvMesh};

/// The mesh's bounds and their centre.
fn bounds(mesh: BUvMesh<'_>) -> ([f32; 3], [f32; 3]) {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in mesh.pos {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    if mesh.pos.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    (lo, hi)
}

/// Project every corner.
pub fn project(mesh: BUvMesh<'_>, how: BUnwrap) -> Vec<[f32; 2]> {
    let (lo, hi) = bounds(mesh);
    let centre = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
    let span = (0..3).map(|k| hi[k] - lo[k]).fold(1e-6f32, f32::max);
    let mut uv = vec![[0.0f32; 2]; mesh.corner.len()];
    for f in 0..mesh.faces() {
        let run = mesh.face(f);
        let (a, b) = (mesh.starts[f] as usize, mesh.starts[f + 1] as usize);
        // Cube projection picks a plane per FACE, from the face's own
        // normal — that is what makes it a cube projection rather than a
        // single flat one.
        let n = face_normal(mesh, run);
        for (k, c) in (a..b).enumerate() {
            let p = mesh.pos[run[k] as usize];
            let q = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
            uv[c] = match how {
                BUnwrap::Cube => {
                    let axis = (0..3).max_by(|x, y| n[*x].abs().total_cmp(&n[*y].abs())).unwrap_or(2);
                    let (i, j) = match axis {
                        0 => (2, 1),
                        1 => (0, 2),
                        _ => (0, 1),
                    };
                    let flip = if n[axis] < 0.0 { -1.0 } else { 1.0 };
                    [q[i] * flip / span + 0.5, q[j] / span + 0.5]
                }
                BUnwrap::Cylinder => {
                    // Around Z, as Blender's default alignment does.
                    let t = q[1].atan2(q[0]);
                    [t / std::f32::consts::TAU + 0.5, q[2] / span + 0.5]
                }
                BUnwrap::Sphere => {
                    let r = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt().max(1e-9);
                    let t = q[1].atan2(q[0]);
                    [t / std::f32::consts::TAU + 0.5, (q[2] / r).clamp(-1.0, 1.0).asin() / std::f32::consts::PI + 0.5]
                }
                BUnwrap::View([right, up]) => {
                    let d = |v: [f32; 3]| q[0] * v[0] + q[1] * v[1] + q[2] * v[2];
                    [d(right) / span + 0.5, d(up) / span + 0.5]
                }
                BUnwrap::Conformal => [0.0; 2],
            };
        }
    }
    uv
}

fn face_normal(mesh: BUvMesh<'_>, run: &[u32]) -> [f32; 3] {
    let mut n = [0.0f32; 3];
    for k in 0..run.len() {
        let (p, q) = (mesh.pos[run[k] as usize], mesh.pos[run[(k + 1) % run.len()] as usize]);
        n[0] += (p[1] - q[1]) * (p[2] + q[2]);
        n[1] += (p[2] - q[2]) * (p[0] + q[0]);
        n[2] += (p[0] - q[0]) * (p[1] + q[1]);
    }
    n
}

#[cfg(test)]
#[path = "test/project.rs"]
mod project_test;
