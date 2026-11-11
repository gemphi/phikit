//! The rounded primitives: sphere, torus, Suzanne, icosahedron, ico sphere, metaball.
use crate::spatial::spatial_object::*;
impl SpatialObject3D {
    pub(super) fn build_sphere_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        // Blender's `Add ▸ Mesh ▸ UV Sphere` default: 32 segments, 16
        // rings. At 12 x 16 the silhouette showed its stacks even with
        // smooth normals — a smooth *shading* cannot hide a coarse edge.
        const STACKS: usize = 16;
        const SLICES: usize = 32;
        let mut verts = Vec::with_capacity((STACKS + 1) * (SLICES + 1));
        for i in 0..=STACKS {
            let v = i as f32 / STACKS as f32;
            let phi = v * std::f32::consts::PI;
            let (sp, cp) = phi.sin_cos();
            for j in 0..=SLICES {
                let u = j as f32 / SLICES as f32;
                let theta = u * std::f32::consts::TAU;
                let (st, ct) = theta.sin_cos();
                verts.push((sp * ct, cp, sp * st));
            }
        }
        let stride = (SLICES + 1) as u32;
        let mut idx = Vec::with_capacity(STACKS * SLICES * 6);
        for i in 0..STACKS {
            for j in 0..SLICES {
                let a = i as u32 * stride + j as u32;
                let b = a + stride;
                idx.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
            }
        }
        let norms = verts.clone();
        (verts, idx, norms)
    }

    /// Torus of major radius 1 and minor radius `TUBE`, generated as a
    /// `RINGS x SIDES` grid. Blender's default `Add ▸ Mesh ▸ Torus` uses
    /// 48x12; 32x16 is plenty at the sizes the CPU rasteriser draws.
    ///
    /// Previously this returned a single triangle, so a torus in the scene
    /// rendered as one flat shard.
    pub(super) fn build_torus_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        const RINGS: usize = 32; // around the main axis
        const SIDES: usize = 16; // around the tube
        const TUBE: f32 = 0.35;

        let mut verts = Vec::with_capacity((RINGS + 1) * (SIDES + 1));
        let mut norms = Vec::with_capacity((RINGS + 1) * (SIDES + 1));
        for i in 0..=RINGS {
            let u = i as f32 / RINGS as f32 * std::f32::consts::TAU;
            let (su, cu) = u.sin_cos();
            for j in 0..=SIDES {
                let v = j as f32 / SIDES as f32 * std::f32::consts::TAU;
                let (sv, cv) = v.sin_cos();
                // Ring centre is on the XZ plane; the tube sweeps in Y.
                let nx = cv * cu;
                let nz = cv * su;
                let ny = sv;
                verts.push((cu + TUBE * nx, TUBE * ny, su + TUBE * nz));
                norms.push((nx, ny, nz));
            }
        }
        let stride = (SIDES + 1) as u32;
        let mut idx = Vec::with_capacity(RINGS * SIDES * 6);
        for i in 0..RINGS as u32 {
            for j in 0..SIDES as u32 {
                let a = i * stride + j;
                let b = a + stride;
                idx.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
            }
        }
        (verts, idx, norms)
    }

    /// Stand-in for Suzanne: a capsule-ish head with a muzzle and two ears,
    /// built from primitives rather than the 507-vertex Blender mesh (which
    /// would have to be vendored as data). Previously this was two triangles.
    ///
    /// Not the real Suzanne — it is a recognisable placeholder volume so the
    /// mesh type is not visually broken. Replace with the imported mesh when
    /// a `.blend` asset is available.
    pub(super) fn build_suzanne_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        let mut verts: Vec<(f32, f32, f32)> = Vec::new();
        let mut norms: Vec<(f32, f32, f32)> = Vec::new();
        let mut idx: Vec<u32> = Vec::new();

        // (centre, radii) for the head, the muzzle and the two ears.
        let parts: [((f32, f32, f32), (f32, f32, f32)); 4] = [
            ((0.00, 0.00, 0.00), (0.85, 0.95, 0.80)),
            ((0.00, -0.25, 0.72), (0.45, 0.35, 0.42)),
            ((-0.92, 0.30, 0.00), (0.22, 0.42, 0.30)),
            ((0.92, 0.30, 0.00), (0.22, 0.42, 0.30)),
        ];
        for (centre, radii) in parts {
            let base = verts.len() as u32;
            const STACKS: usize = 10;
            const SLICES: usize = 14;
            for i in 0..=STACKS {
                let phi = i as f32 / STACKS as f32 * std::f32::consts::PI;
                let (sp, cp) = phi.sin_cos();
                for j in 0..=SLICES {
                    let theta = j as f32 / SLICES as f32 * std::f32::consts::TAU;
                    let (st, ct) = theta.sin_cos();
                    let n = (sp * ct, cp, sp * st);
                    verts.push((
                        centre.0 + n.0 * radii.0,
                        centre.1 + n.1 * radii.1,
                        centre.2 + n.2 * radii.2,
                    ));
                    norms.push(n);
                }
            }
            let stride = (SLICES + 1) as u32;
            for i in 0..STACKS as u32 {
                for j in 0..SLICES as u32 {
                    let a = base + i * stride + j;
                    let b = a + stride;
                    idx.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
                }
            }
        }
        (verts, idx, norms)
    }

    /// A real icosahedron: the twelve vertices of (0, ±1, ±φ) and its
    /// permutations, and the **twenty** faces that actually join them,
    /// wound outward.
    ///
    /// What was here before was neither. Of its twenty triangles,
    /// eighteen were wrong: most joined three vertices that are not
    /// mutually adjacent — not faces of the solid at all — and all but
    /// three were wound inward. On screen that is not a gem with a bad
    /// normal somewhere; it is a shattered mess of interpenetrating
    /// shards, which is exactly what it looked like. The faces are
    /// derived rather than typed: a triple is a face precisely when its
    /// three edges are all the solid's own edge length, and it is wound
    /// so its normal points away from the centre.
    pub(super) fn build_icosahedron_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        let (verts, idx) = Self::icosahedron();
        // Smooth: on a unit sphere the vertex normal *is* the position.
        let norms = verts.clone();
        (verts, idx, norms)
    }

    /// The twelve vertices and twenty outward faces of a unit icosahedron.
    fn icosahedron() -> (Vec<(f32, f32, f32)>, Vec<u32>) {
        let phi: f32 = (1.0 + 5.0f32.sqrt()) / 2.0;
        let inv = 1.0 / (1.0 + phi * phi).sqrt();
        let (a, b) = (inv, phi * inv);
        let verts = vec![
            (0.0, a, b),
            (0.0, a, -b),
            (0.0, -a, b),
            (0.0, -a, -b),
            (a, b, 0.0),
            (a, -b, 0.0),
            (-a, b, 0.0),
            (-a, -b, 0.0),
            (b, 0.0, a),
            (-b, 0.0, a),
            (b, 0.0, -a),
            (-b, 0.0, -a),
        ];
        let idx: Vec<u32> = vec![
            0, 2, 8, 0, 9, 2, 0, 4, 6, 0, 8, 4, 0, 6, 9, 1, 10, 3, 1, 3, 11, 1, 6, 4, 1, 4, 10, 1,
            11, 6, 2, 7, 5, 2, 5, 8, 2, 9, 7, 3, 5, 7, 3, 10, 5, 3, 7, 11, 4, 8, 10, 5, 10, 8, 6,
            11, 9, 7, 9, 11,
        ];
        (verts, idx)
    }

    /// Blender's `Add ▸ Mesh ▸ Ico Sphere`: the icosahedron subdivided
    /// `subdivisions` times with every new vertex pushed back onto the
    /// sphere, so the triangles stay near-equilateral where a UV sphere's
    /// crowd at the poles. Blender's default is 2 subdivisions.
    pub(super) fn build_ico_sphere_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        let (mut verts, mut idx) = Self::icosahedron();
        for _ in 0..2 {
            let mut out = Vec::with_capacity(idx.len() * 4);
            // Each edge's midpoint is made once and shared, or the sphere
            // comes apart into unwelded triangles at every seam.
            let mut mid: std::collections::HashMap<(u32, u32), u32> = std::collections::HashMap::new();
            let mut midpoint = |verts: &mut Vec<(f32, f32, f32)>, p: u32, q: u32| -> u32 {
                let key = if p < q { (p, q) } else { (q, p) };
                if let Some(&k) = mid.get(&key) {
                    return k;
                }
                let (u, v) = (verts[p as usize], verts[q as usize]);
                let m = (u.0 + v.0, u.1 + v.1, u.2 + v.2);
                let len = (m.0 * m.0 + m.1 * m.1 + m.2 * m.2).sqrt().max(1e-9);
                verts.push((m.0 / len, m.1 / len, m.2 / len));
                let k = verts.len() as u32 - 1;
                mid.insert(key, k);
                k
            };
            for t in idx.chunks_exact(3) {
                let (a, b, c) = (t[0], t[1], t[2]);
                let (ab, bc, ca) = (midpoint(&mut verts, a, b), midpoint(&mut verts, b, c), midpoint(&mut verts, c, a));
                out.extend_from_slice(&[a, ab, ca, b, bc, ab, c, ca, bc, ab, bc, ca]);
            }
            idx = out;
        }
        let norms = verts.clone();
        (verts, idx, norms)
    }

    /// Blender's `Add ▸ Metaball ▸ Ball`: three blobs that fuse into one
    /// surface, polygonised at display time like Blender's own
    /// (`spatial/implicit.rs`).
    pub(super) fn build_metaball_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        use crate::spatial::implicit::{Blob, polygonise};
        let blobs = [
            Blob { at: [0.0, 0.0, 0.0], radius: 0.58 },
            Blob { at: [0.62, 0.42, 0.0], radius: 0.40 },
            Blob { at: [-0.58, -0.34, 0.26], radius: 0.36 },
        ];
        polygonise(&blobs, 1.0, 1.6, 30)
    }

}
