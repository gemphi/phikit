//! The projections: not solvers at all, which is why they are instant and
//! why they distort where the surface turns away from the axis.
//!
//! Blender's `uv.cube_project`, `uv.cylinder_project`, `uv.sphere_project`
//! and `uv.project_from_view`. Each is the same shape of thing — a point
//! in space mapped to a point in the square — and the only question is
//! which point.

use super::{BUnwrap, BUvMesh};

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
mod tests {
    use super::*;

    /// A unit cube's six faces, each its own quad.
    fn cube() -> (Vec<[f32; 3]>, Vec<u32>, Vec<u32>) {
        let p = [
            [-1.0, -1.0, -1.0], [1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0],
        ];
        let f: [[u32; 4]; 6] = [[0, 3, 2, 1], [4, 5, 6, 7], [0, 1, 5, 4], [2, 3, 7, 6], [1, 2, 6, 5], [0, 4, 7, 3]];
        let (mut corner, mut starts) = (Vec::new(), vec![0u32]);
        for q in f {
            corner.extend_from_slice(&q);
            starts.push(corner.len() as u32);
        }
        (p.to_vec(), corner, starts)
    }

    /// Cube projection puts every face square in the square: each face's
    /// four UVs span the same box, because each is seen head on.
    #[test]
    fn cube_project_lays_each_face_flat() {
        let (pos, corner, starts) = cube();
        let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
        let uv = project(mesh, BUnwrap::Cube);
        for f in 0..6 {
            let (a, b) = (starts[f] as usize, starts[f + 1] as usize);
            let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
            for c in a..b {
                for k in 0..2 {
                    lo[k] = lo[k].min(uv[c][k]);
                    hi[k] = hi[k].max(uv[c][k]);
                }
            }
            assert!((hi[0] - lo[0] - 1.0).abs() < 1e-5 && (hi[1] - lo[1] - 1.0).abs() < 1e-5, "face {f} is a unit square: {lo:?} {hi:?}");
        }
    }

    /// Cylinder projection turns the angle into U, so a ring of points
    /// around Z comes out spread across U and flat in V.
    #[test]
    fn cylinder_project_unrolls_the_angle() {
        let seg = 8;
        let mut pos = Vec::new();
        for i in 0..seg {
            let t = i as f32 / seg as f32 * std::f32::consts::TAU;
            pos.push([t.cos(), t.sin(), -1.0]);
            pos.push([t.cos(), t.sin(), 1.0]);
        }
        let (mut corner, mut starts) = (Vec::new(), vec![0u32]);
        for i in 0..seg as u32 {
            let j = (i + 1) % seg as u32;
            corner.extend_from_slice(&[2 * i, 2 * j, 2 * j + 1, 2 * i + 1]);
            starts.push(corner.len() as u32);
        }
        let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
        let uv = project(mesh, BUnwrap::Cylinder);
        let (lo, hi) = uv.iter().fold((1.0f32, 0.0f32), |(l, h), c| (l.min(c[0]), h.max(c[0])));
        assert!(hi - lo > 0.8, "U runs most of the way round: {lo}..{hi}");
        // The two rings are at the two ends of V and nowhere between.
        let vs: Vec<f32> = uv.iter().map(|c| c[1]).collect();
        assert!(vs.iter().all(|v| (*v - 0.0).abs() < 1e-5 || (*v - 1.0).abs() < 1e-5), "V is the height: {vs:?}");
    }
}
