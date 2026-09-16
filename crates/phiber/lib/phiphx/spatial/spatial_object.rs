//! Native Rust 3D Spatial Object & Geometry Mesh Buffer Representation

use crate::phiphx::spatial::mesh_converter::PhiQuiMeshType;
use crate::phiphx::spatial::spatial_animator::SpatialTransform3D;

/// Compute a smooth per-vertex normal buffer from a triangle-indexed mesh.
///
/// Each triangle's geometric (face) normal is accumulated onto its three
/// vertices, then every vertex normal is normalized. Returns one normal per
/// vertex (same length as `verts`), matching the convention used by the other
/// primitive builders in this module.
fn face_normals(verts: &[(f32, f32, f32)], idx: &[u32]) -> Vec<(f32, f32, f32)> {
    let mut acc = vec![(0.0f32, 0.0f32, 0.0f32); verts.len()];
    for tri in idx.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let (pa, pb, pc) = (verts[a], verts[b], verts[c]);
        let e1 = (pb.0 - pa.0, pb.1 - pa.1, pb.2 - pa.2);
        let e2 = (pc.0 - pa.0, pc.1 - pa.1, pc.2 - pa.2);
        // Cross product e1 × e2 — the (unnormalized) face normal.
        let n = (
            e1.1 * e2.2 - e1.2 * e2.1,
            e1.2 * e2.0 - e1.0 * e2.2,
            e1.0 * e2.1 - e1.1 * e2.0,
        );
        for &v in &[a, b, c] {
            acc[v].0 += n.0;
            acc[v].1 += n.1;
            acc[v].2 += n.2;
        }
    }
    for n in acc.iter_mut() {
        let len = (n.0 * n.0 + n.1 * n.1 + n.2 * n.2).sqrt();
        if len > f32::EPSILON {
            n.0 /= len;
            n.1 /= len;
            n.2 /= len;
        }
    }
    acc
}

#[derive(Debug, Clone)]
pub struct SpatialObject3D {
    pub name: String,
    pub mesh_type: PhiQuiMeshType,
    pub transform: SpatialTransform3D,
    pub vertices: Vec<(f32, f32, f32)>,
    pub indices: Vec<u32>,
    pub normals: Vec<(f32, f32, f32)>,
    pub color: (f32, f32, f32, f32),
}

impl SpatialObject3D {
    pub fn new(name: impl Into<String>, mesh_type: PhiQuiMeshType) -> Self {
        let (vertices, indices, normals) = Self::buffers(mesh_type);
        Self {
            name: name.into(),
            mesh_type,
            transform: SpatialTransform3D::default(),
            vertices,
            indices,
            normals,
            color: (0.65, 0.33, 0.96, 1.0),
        }
    }

    pub fn with_transform(mut self, transform: SpatialTransform3D) -> Self {
        self.transform = transform;
        self
    }

    /// Vertex / triangle / normal buffers for a primitive. Cube is unit,
    /// sphere is a UV sphere, icosahedron is a normalized gem.
    pub fn buffers(
        mesh_type: PhiQuiMeshType,
    ) -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        match mesh_type {
            PhiQuiMeshType::Cube => Self::build_cube_mesh(),
            PhiQuiMeshType::Sphere => Self::build_sphere_mesh(),
            PhiQuiMeshType::Torus => Self::build_torus_mesh(),
            PhiQuiMeshType::Suzanne => Self::build_suzanne_mesh(),
            PhiQuiMeshType::Cylinder => Self::build_cylinder_mesh(),
            PhiQuiMeshType::Icosahedron => Self::build_icosahedron_mesh(),
            PhiQuiMeshType::Plane => Self::build_plane_mesh(),
            PhiQuiMeshType::Circle => Self::build_circle_mesh(),
            PhiQuiMeshType::Cone => Self::build_cone_mesh(),
        }
    }

    fn build_cube_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
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

    fn build_sphere_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        const STACKS: usize = 12;
        const SLICES: usize = 16;
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
    fn build_torus_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
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
    fn build_suzanne_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
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

    /// Closed cylinder: a `SEGMENTS`-sided tube with flat caps, unit radius
    /// and height 2 (matching the cube's -1..1 extent). Previously a quad.
    fn build_cylinder_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
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

    /// Flat unit quad on the XZ plane, y=0 — Blender's `Add ▸ Mesh ▸ Plane`.
    /// Two triangles, normal pointing +Y.
    fn build_plane_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
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

    /// Regular polygon on the XZ plane — Blender's `Add ▸ Mesh ▸ Circle`.
    /// 32 rim segments + centre vertex; triangulated as a fan. Normal +Y.
    fn build_circle_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
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

    /// Closed right-cone — Blender's `Add ▸ Mesh ▸ Cone`. 32 base segments,
    /// apex at (0, 1, 0), base rim at y=-1 with radius 1, plus a base fan
    /// closing the bottom.
    fn build_cone_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
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

    fn build_icosahedron_mesh() -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        // 12 vertices of a unit icosahedron: (0, ±1, ±φ), (±1, ±φ, 0), (±φ, 0, ±1).
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
            0, 6, 1, 0, 11, 6, 1, 4, 0, 1, 8, 4, 1, 10, 8, 2, 5, 3, 2, 8, 5, 2, 10, 11, 3, 7, 6, 3,
            9, 7, 4, 10, 5, 4, 8, 0, 5, 8, 2, 6, 7, 1, 9, 11, 7, 9, 3, 6, 10, 11, 2, 10, 0, 8, 11,
            3, 6, 5, 4, 9,
        ];
        let norms = verts.clone(); // unit sphere ⇒ vertex normal == position
        (verts, idx, norms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::spatial::mesh_converter::PhiQuiMeshType;

    #[test]
    fn cube_has_twelve_triangles() {
        let (v, i, n) = SpatialObject3D::buffers(PhiQuiMeshType::Cube);
        assert_eq!(v.len(), 24);
        assert_eq!(i.len(), 36);
        assert_eq!(n.len(), 24);
    }

    #[test]
    fn sphere_is_uv_grid() {
        let (v, i, n) = SpatialObject3D::buffers(PhiQuiMeshType::Sphere);
        assert_eq!(v.len(), 13 * 17);
        assert_eq!(i.len(), 12 * 16 * 6);
        assert_eq!(n.len(), v.len());
    }

    #[test]
    fn plane_has_two_triangles() {
        let (v, i, n) = SpatialObject3D::buffers(PhiQuiMeshType::Plane);
        assert_eq!(v.len(), 4);
        assert_eq!(i.len(), 6);
        assert_eq!(n.len(), 4);
        // All normals point +Y for a ground-plane quad.
        for &(_, ny, _) in &n {
            assert!((ny - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn circle_has_32_segment_fan() {
        let (v, i, n) = SpatialObject3D::buffers(PhiQuiMeshType::Circle);
        assert_eq!(v.len(), 33); // 32 rim + centre
        assert_eq!(i.len(), 32 * 3);
        assert_eq!(n.len(), v.len());
        // Every rim vertex lies on the unit circle in XZ.
        for &(x, _, z) in &v[..32] {
            let r = (x * x + z * z).sqrt();
            assert!((r - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn cone_has_side_and_base() {
        let (v, i, _n) = SpatialObject3D::buffers(PhiQuiMeshType::Cone);
        // 32 side faces (apex + 2 rim each) + 1 base centre + 32 base rim.
        assert_eq!(v.len(), 32 * 3 + 1 + 32);
        // 32 side triangles + 32 base triangles.
        assert_eq!(i.len(), 32 * 3 + 32 * 3);
    }

    #[test]
    fn icosahedron_is_twenty_faces() {
        let (v, i, n) = SpatialObject3D::buffers(PhiQuiMeshType::Icosahedron);
        assert_eq!(v.len(), 12);
        assert_eq!(i.len(), 60);
        assert_eq!(n.len(), 12);
        for &(x, y, z) in &v {
            let len = (x * x + y * y + z * z).sqrt();
            assert!((len - 1.0).abs() < 1e-5);
        }
    }
}
