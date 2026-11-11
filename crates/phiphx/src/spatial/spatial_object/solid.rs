//! The flat-sided and extruded primitives: cube, plane, grid, circle, cylinder, cone.
use super::*;
impl SpatialObject3D {
    pub(super) fn build_cube_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        // Unique verts per face so averaged normals stay faceted.
        let faces = [
            [
                (-1.0, -1.0, 1.0),
                (1.0, -1.0, 1.0),
                (1.0, 1.0, 1.0),
                (-1.0, 1.0, 1.0),
            ],
            [
                (1.0, -1.0, -1.0),
                (-1.0, -1.0, -1.0),
                (-1.0, 1.0, -1.0),
                (1.0, 1.0, -1.0),
            ],
            [
                (1.0, -1.0, 1.0),
                (1.0, -1.0, -1.0),
                (1.0, 1.0, -1.0),
                (1.0, 1.0, 1.0),
            ],
            [
                (-1.0, -1.0, -1.0),
                (-1.0, -1.0, 1.0),
                (-1.0, 1.0, 1.0),
                (-1.0, 1.0, -1.0),
            ],
            [
                (-1.0, 1.0, 1.0),
                (1.0, 1.0, 1.0),
                (1.0, 1.0, -1.0),
                (-1.0, 1.0, -1.0),
            ],
            [
                (-1.0, -1.0, -1.0),
                (1.0, -1.0, -1.0),
                (1.0, -1.0, 1.0),
                (-1.0, -1.0, 1.0),
            ],
        ];
        let mut verts = Vec::with_capacity(24);
        let mut idx = Vec::with_capacity(36);
        for face in faces {
            let base = verts.len() as u32;
            verts.extend_from_slice(&face);
            idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        let norms = face_normals(&verts, &idx);
        (verts, idx, norms)
    }

    /// Flat unit quad on the XZ plane, y=0 — Blender's `Add ▸ Mesh ▸ Plane`.
    /// Two triangles, normal pointing +Y.
    pub(super) fn build_plane_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        let verts = vec![
            (-1.0, 0.0, -1.0),
            (1.0, 0.0, -1.0),
            (1.0, 0.0, 1.0),
            (-1.0, 0.0, 1.0),
        ];
        let idx = vec![0, 2, 1, 0, 3, 2];
        let norms = vec![(0.0, 1.0, 0.0); 4];
        (verts, idx, norms)
    }

    /// Blender's `Add ▸ Mesh ▸ Grid`: a flat `N x N` quad grid on the
    /// ground plane, where Plane is the same thing at one quad.
    pub(super) fn build_grid_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        const N: usize = 10;
        let mut verts = Vec::with_capacity((N + 1) * (N + 1));
        for i in 0..=N {
            let z = i as f32 / N as f32 * 2.0 - 1.0;
            for j in 0..=N {
                let x = j as f32 / N as f32 * 2.0 - 1.0;
                verts.push((x, 0.0, z));
            }
        }
        let stride = (N + 1) as u32;
        let mut idx = Vec::with_capacity(N * N * 6);
        for i in 0..N as u32 {
            for j in 0..N as u32 {
                let a = i * stride + j;
                idx.extend_from_slice(&[a, a + stride, a + 1, a + 1, a + stride, a + stride + 1]);
            }
        }
        let norms = vec![(0.0, 1.0, 0.0); verts.len()];
        (verts, idx, norms)
    }
    /// Regular polygon on the XZ plane — Blender's `Add ▸ Mesh ▸ Circle`.
    /// 32 rim segments + centre vertex; triangulated as a fan. Normal +Y.
    pub(super) fn build_circle_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        const SEGMENTS: usize = 32;
        let mut verts: Vec<(f32, f32, f32)> = Vec::with_capacity(SEGMENTS + 1);
        // Rim first, then centre.
        for j in 0..SEGMENTS {
            let theta = j as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let (st, ct) = theta.sin_cos();
            verts.push((ct, 0.0, st));
        }
        let centre = verts.len() as u32;
        verts.push((0.0, 0.0, 0.0));

        let mut idx: Vec<u32> = Vec::with_capacity(SEGMENTS * 3);
        for j in 0..SEGMENTS as u32 {
            let next = (j + 1) % SEGMENTS as u32;
            // Wind so the normal points +Y (upward).
            idx.extend_from_slice(&[centre, next, j]);
        }
        let norms = vec![(0.0, 1.0, 0.0); verts.len()];
        (verts, idx, norms)
    }

    /// Closed cylinder: a `SEGMENTS`-sided tube with flat caps, unit radius
    /// and height 2 (matching the cube's -1..1 extent). Previously a quad.
    pub(super) fn build_cylinder_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        const SEGMENTS: usize = 24;
        let mut verts: Vec<(f32, f32, f32)> = Vec::new();
        let mut norms: Vec<(f32, f32, f32)> = Vec::new();
        let mut idx: Vec<u32> = Vec::new();

        // Side wall — duplicated rim vertices so the caps stay flat-shaded.
        for j in 0..=SEGMENTS {
            let theta = j as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let (st, ct) = theta.sin_cos();
            verts.push((ct, 1.0, st));
            norms.push((ct, 0.0, st));
            verts.push((ct, -1.0, st));
            norms.push((ct, 0.0, st));
        }
        for j in 0..SEGMENTS as u32 {
            let a = j * 2;
            idx.extend_from_slice(&[a, a + 1, a + 2, a + 2, a + 1, a + 3]);
        }

        // Caps: one centre vertex plus a fan of rim vertices each.
        for (y, ny) in [(1.0f32, 1.0f32), (-1.0, -1.0)] {
            let centre = verts.len() as u32;
            verts.push((0.0, y, 0.0));
            norms.push((0.0, ny, 0.0));
            let rim = verts.len() as u32;
            for j in 0..=SEGMENTS {
                let theta = j as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
                let (st, ct) = theta.sin_cos();
                verts.push((ct, y, st));
                norms.push((0.0, ny, 0.0));
            }
            for j in 0..SEGMENTS as u32 {
                // Wind the bottom cap the other way so both face outward.
                if ny > 0.0 {
                    idx.extend_from_slice(&[centre, rim + j, rim + j + 1]);
                } else {
                    idx.extend_from_slice(&[centre, rim + j + 1, rim + j]);
                }
            }
        }
        (verts, idx, norms)
    }

    /// Closed right-cone — Blender's `Add ▸ Mesh ▸ Cone`. 32 base segments,
    /// apex at (0, 1, 0), base rim at y=-1 with radius 1, plus a base fan
    /// closing the bottom.
    pub(super) fn build_cone_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        const SEGMENTS: usize = 32;
        let mut verts: Vec<(f32, f32, f32)> = Vec::new();
        let mut norms: Vec<(f32, f32, f32)> = Vec::new();
        let mut idx: Vec<u32> = Vec::new();

        // Side wall: for each segment we emit apex + two rim verts so the
        // normal is per-face and points outward.
        for j in 0..SEGMENTS {
            let theta0 = j as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let theta1 = (j + 1) as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let (s0, c0) = theta0.sin_cos();
            let (s1, c1) = theta1.sin_cos();
            let apex = (0.0, 1.0, 0.0);
            let r0 = (c0, -1.0, s0);
            let r1 = (c1, -1.0, s1);
            // Face normal from cross product of two edges.
            let e1 = (r0.0 - apex.0, r0.1 - apex.1, r0.2 - apex.2);
            let e2 = (r1.0 - apex.0, r1.1 - apex.1, r1.2 - apex.2);
            let n = (
                e1.1 * e2.2 - e1.2 * e2.1,
                e1.2 * e2.0 - e1.0 * e2.2,
                e1.0 * e2.1 - e1.1 * e2.0,
            );
            let len = (n.0 * n.0 + n.1 * n.1 + n.2 * n.2).sqrt().max(1e-8);
            let n = (n.0 / len, n.1 / len, n.2 / len);
            let base = verts.len() as u32;
            verts.push(apex);
            verts.push(r0);
            verts.push(r1);
            norms.push(n);
            norms.push(n);
            norms.push(n);
            idx.extend_from_slice(&[base, base + 1, base + 2]);
        }

        // Base fan: centre + rim, wound so normal is -Y.
        let centre = verts.len() as u32;
        verts.push((0.0, -1.0, 0.0));
        norms.push((0.0, -1.0, 0.0));
        let rim_start = verts.len() as u32;
        for j in 0..SEGMENTS {
            let theta = j as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let (st, ct) = theta.sin_cos();
            verts.push((ct, -1.0, st));
            norms.push((0.0, -1.0, 0.0));
        }
        for j in 0..SEGMENTS as u32 {
            let a = rim_start + j;
            let b = rim_start + (j + 1) % SEGMENTS as u32;
            idx.extend_from_slice(&[centre, a, b]);
        }

        (verts, idx, norms)
    }

}
