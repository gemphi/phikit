use crate::spatial::spatial_object::*;
use crate::spatial::mesh_converter::PhiQuiMeshType;

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
    assert_eq!(v.len(), 17 * 33);
    assert_eq!(i.len(), 16 * 32 * 6);
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

/// Counting the triangles is not the test. The old gem had exactly
/// twelve vertices, exactly twenty triangles and every vertex on the
/// unit sphere — and eighteen of its twenty triangles were not faces of
/// the solid at all, so it drew as a shattered mess. What a solid has
/// to satisfy: every triangle's three edges are the solid's own edge
/// length, every triangle faces away from the centre, and every edge is
/// shared by exactly two triangles.
#[test]
fn the_gem_is_a_solid_and_not_a_pile_of_shards() {
    let (v, i, _) = SpatialObject3D::buffers(PhiQuiMeshType::Icosahedron);
    let dist = |a: usize, b: usize| {
        let (p, q) = (v[a], v[b]);
        ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2) + (p.2 - q.2).powi(2)).sqrt()
    };
    // The solid's own edge: the shortest distance between any two of
    // its vertices.
    let mut edge = f32::MAX;
    for a in 0..v.len() {
        for b in a + 1..v.len() {
            edge = edge.min(dist(a, b));
        }
    }
    let mut seen: std::collections::HashMap<(u32, u32), usize> = std::collections::HashMap::new();
    for t in i.chunks_exact(3) {
        let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
        for (p, q) in [(a, b), (b, c), (c, a)] {
            assert!((dist(p, q) - edge).abs() < 1e-4, "{p}-{q} is an edge of the solid, not a chord across it");
            *seen.entry((p.min(q) as u32, p.max(q) as u32)).or_default() += 1;
        }
        // Outward: the face normal agrees with the direction from the
        // centre to the face.
        let (pa, pb, pc) = (v[a], v[b], v[c]);
        let u = (pb.0 - pa.0, pb.1 - pa.1, pb.2 - pa.2);
        let w = (pc.0 - pa.0, pc.1 - pa.1, pc.2 - pa.2);
        let nrm = (u.1 * w.2 - u.2 * w.1, u.2 * w.0 - u.0 * w.2, u.0 * w.1 - u.1 * w.0);
        let mid = ((pa.0 + pb.0 + pc.0) / 3.0, (pa.1 + pb.1 + pc.1) / 3.0, (pa.2 + pb.2 + pc.2) / 3.0);
        assert!(nrm.0 * mid.0 + nrm.1 * mid.1 + nrm.2 * mid.2 > 0.0, "the face {a},{b},{c} is wound outward");
    }
    assert_eq!(seen.len(), 30, "an icosahedron has thirty edges");
    assert!(seen.values().all(|&n| n == 2), "every edge is shared by exactly two faces");
}

/// The Ico Sphere is the gem subdivided onto the sphere: every vertex
/// on the unit sphere, four times the triangles per subdivision, and
/// the midpoints shared rather than duplicated — an unwelded sphere
/// would have 4x the vertices and split shading at every seam.
#[test]
fn the_ico_sphere_is_welded_and_round() {
    let (v, i, n) = SpatialObject3D::buffers(PhiQuiMeshType::IcoSphere);
    assert_eq!(i.len() / 3, 20 * 4 * 4, "two subdivisions");
    // Euler: V - E + F = 2 with F = 320 and E = 3F/2 gives V = 162.
    assert_eq!(v.len(), 162, "the midpoints are shared");
    assert_eq!(n.len(), v.len());
    for &(x, y, z) in &v {
        assert!(((x * x + y * y + z * z).sqrt() - 1.0).abs() < 1e-5);
    }
}

/// A metaball is a surface, not a cloud: it has triangles, they are
/// all within the sampled box, and nothing is degenerate.
#[test]
fn the_metaball_is_a_closed_surface() {
    let (v, i, n) = SpatialObject3D::buffers(PhiQuiMeshType::Metaball);
    assert!(i.len() >= 300 && i.len() % 3 == 0, "{} indices", i.len());
    assert_eq!(n.len(), v.len());
    for &(x, y, z) in &v {
        assert!(x.abs() <= 1.7 && y.abs() <= 1.7 && z.abs() <= 1.7, "inside the sampled box");
    }
    for &(nx, ny, nz) in &n {
        assert!(((nx * nx + ny * ny + nz * nz).sqrt() - 1.0).abs() < 1e-3, "unit normals");
    }
}
