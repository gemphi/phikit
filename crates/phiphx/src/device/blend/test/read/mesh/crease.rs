use crate::device::blend::read::mesh::crease::edge_verts_are_good;

/// Blender's `find_edges_bad_verts`, rule for rule. Before this, an edge
/// naming vertex 24 of a 24-vertex mesh reached the modifier stack and
/// panicked the Mirror modifier building its vertex map
/// (`modifier/mirror.rs`, `the len is 24 but the index is 24`) and
/// Subsurf blending its midpoint's group weights (`modifier/subsurf.rs`).
#[test]
fn an_edge_is_kept_unless_blender_would_have_dropped_it() {
    // In range and distinct: kept, either way round.
    assert!(edge_verts_are_good(0, 23, 24));
    assert!(edge_verts_are_good(23, 0, 24));
    // "Edge {} has out of range vertex": one past the last vertex is the
    // case that panicked, and so is anything beyond it.
    assert!(!edge_verts_are_good(0, 24, 24));
    assert!(!edge_verts_are_good(24, 0, 24));
    assert!(!edge_verts_are_good(0, 3_187_833_393, 24));
    // "Edge {} has equal vertex indices": a loop on one vertex.
    assert!(!edge_verts_are_good(7, 7, 24));
    // A mesh with no vertices keeps no edges.
    assert!(!edge_verts_are_good(0, 0, 0));
}
