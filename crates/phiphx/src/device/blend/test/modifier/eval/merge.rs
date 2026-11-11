//! Displace and Weld: modifiers that move or remove vertices.

use super::super::displace::BDisplace;
use super::super::weld::BWeld;
use super::{eval, quad};
use super::super::BlendEvalMesh;

#[test]
fn displace_moves_along_an_axis_by_strength_above_midlevel() {
    let mut m = quad();
    eval(&mut m, &BDisplace { strength: 2.0, midlevel: 0.5, direction: 2 });
    // Textureless: every vertex moves strength * (1 - midlevel) along Z.
    for v in &m.vertices {
        assert!((v[2] - 1.0).abs() < 1e-6, "{v:?}");
    }
}

#[test]
fn displace_along_normals_follows_the_surface() {
    let mut m = quad();
    eval(&mut m, &BDisplace { strength: 1.0, midlevel: 0.0, direction: 3 });
    // The quad's normal is +Z, so the whole face lifts by one unit.
    assert!(m.vertices.iter().all(|v| (v[2] - 1.0).abs() < 1e-6), "{:?}", m.vertices);
}

#[test]
fn displace_at_midlevel_is_a_no_op() {
    let mut m = quad();
    let before = m.clone();
    eval(&mut m, &BDisplace { strength: 3.0, midlevel: 1.0, direction: 2 });
    assert_eq!(m, before);
}

// ── Weld ──────────────────────────────────────────────────────────────────

/// Two quads sharing an edge, but with the shared vertices duplicated — the
/// split-seam case Weld exists for.
fn split_seam() -> BlendEvalMesh {
    BlendEvalMesh {
        vertices: vec![
            [0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.],
            [1., 0., 0.], [2., 0., 0.], [2., 1., 0.], [1., 1., 0.],
        ],
        faces: [vec![0u32, 1, 2, 3], vec![4, 5, 6, 7]].iter().cloned().collect(),
        face_materials: vec![0, 1],
        face_smooth: vec![true, true],
        ..Default::default()
    }
}

#[test]
fn weld_joins_coincident_vertices_and_keeps_both_faces() {
    let mut m = split_seam();
    eval(&mut m, &BWeld { merge_dist: 0.001, mode: 0 });
    assert_eq!(m.vertices.len(), 6);
    assert_eq!(m.faces.len(), 2);
    // The second quad now indexes the first's vertices along the seam.
    assert_eq!(m.faces.get(1), &[1, 4, 5, 2]);
    assert_eq!(m.face_materials, vec![0, 1]);
}

#[test]
fn weld_collapses_a_face_that_loses_its_corners() {
    let mut m = quad();
    // Wide enough to pull the whole quad onto a single vertex.
    eval(&mut m, &BWeld { merge_dist: 10.0, mode: 0 });
    assert_eq!(m.vertices.len(), 1);
    assert_eq!(m.faces.len(), 0);
}

#[test]
fn weld_connected_only_merges_across_an_existing_edge() {
    let mut m = split_seam();
    // The duplicated seam vertices are coincident but share no edge, so
    // connected mode leaves them alone.
    eval(&mut m, &BWeld { merge_dist: 0.001, mode: 1 });
    assert_eq!(m.vertices.len(), 8);
    assert_eq!(m.faces.len(), 2);
}
