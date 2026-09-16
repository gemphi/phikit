//! Projection + viewport-fit tests. Split from `mod.rs` so all test files
//! stay under the 200-line cap.

use super::*;

#[test]
fn project_origin_is_near_center() {
    let (x, y) = project(0.0, 0.0, 0.0);
    assert!(x.abs() < 1e-6);
    assert!((y + 0.04).abs() < 1e-6);
}

#[test]
fn project_is_rotation_stable_at_360() {
    let a = project(0.5, 0.3, 0.0);
    let b = project(0.5, 0.3, 360.0);
    assert!((a.0 - b.0).abs() < 1e-4);
    assert!((a.1 - b.1).abs() < 1e-4);
}

#[test]
fn project_zoom_scales_offset() {
    let (bx, by) = project(0.5, 0.3, 45.0);
    let (zx, zy) = project_zoom(0.5, 0.3, 45.0, 2.0);
    assert!((zx - bx * 2.0).abs() < 1e-5);
    assert!((zy - by * 2.0).abs() < 1e-5);
}

#[test]
fn project3_ground_matches_project_zoom() {
    let (sx, sy) = project_zoom(0.5, 0.3, 45.0, 2.0);
    let (px, py, _) = project3(0.5, 0.0, 0.3, 45.0, 2.0);
    assert!((sx - px).abs() < 1e-5);
    assert!((sy - py).abs() < 1e-5);
}

#[test]
fn project3_up_moves_toward_top() {
    let (_, y0, _) = project3(0.0, 0.0, 0.0, 30.0, 1.0);
    let (_, y1, _) = project3(0.0, 1.0, 0.0, 30.0, 1.0);
    assert!(y1 < y0);
}

// ── Viewport-fit transform (task 5.3) ─────────────────────────────────

/// The eight corners of a unit cube spanning `[-0.5, 0.5]` on each axis,
/// centred on the origin.
fn unit_cube() -> Vec<[f32; 3]> {
    vec![
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [0.5, 0.5, 0.5],
    ]
}

fn within_bounds(v: [f32; 3]) -> bool {
    let eps = 1e-5;
    v.iter()
        .all(|&c| c >= -MESH_SCALE - eps && c <= MESH_SCALE + eps)
}

#[test]
fn mesh_bounds_unit_cube_centre_and_radius() {
    let (center, r) = mesh_bounds(&unit_cube());
    assert!(center.iter().all(|&c| c.abs() < 1e-6));
    assert!((r - 0.5).abs() < 1e-6);
}

#[test]
fn fit_unit_cube_vertices_within_mesh_scale_bounds() {
    let fitted = fit_vertices(&unit_cube());
    for v in fitted {
        assert!(
            within_bounds(v),
            "vertex {v:?} escaped [-MESH_SCALE, MESH_SCALE]"
        );
    }
}

#[test]
fn fit_large_offcenter_mesh_fits_same_bounds() {
    // A mesh far from the origin, spanning 10 units on each axis.
    let base = 1000.0_f32;
    let span = 10.0_f32;
    let mut verts = Vec::new();
    for &dx in &[0.0, span] {
        for &dy in &[0.0, span] {
            for &dz in &[0.0, span] {
                verts.push([base + dx, base + dy, base + dz]);
            }
        }
    }
    let (center, r) = mesh_bounds(&verts);
    // Centre is removed (the AABB midpoint), radius is span/2.
    assert!((center[0] - (base + span / 2.0)).abs() < 1e-3);
    assert!((r - span / 2.0).abs() < 1e-3);
    for v in fit_vertices(&verts) {
        assert!(within_bounds(v), "off-center vertex {v:?} escaped bounds");
    }
}

#[test]
fn fit_single_point_mesh_does_not_panic_and_is_finite() {
    // Degenerate: all vertices coincide -> zero extent -> r guard = 1.0.
    let verts = vec![[3.0, -7.0, 42.0]; 4];
    let (center, r) = mesh_bounds(&verts);
    assert!(
        (r - 1.0).abs() < 1e-6,
        "degenerate mesh should fall back to r = 1.0"
    );
    assert!((center[0] - 3.0).abs() < 1e-6);
    let fitted = fit_vertices(&verts);
    for v in fitted {
        assert!(
            v.iter().all(|c| c.is_finite()),
            "fitted vertex not finite: {v:?}"
        );
        // All coincident points map onto the origin after centring.
        assert!(v.iter().all(|&c| c.abs() < 1e-6));
    }
}

#[test]
fn fit_empty_mesh_returns_default_bounds() {
    let (center, r) = mesh_bounds(&[]);
    assert_eq!(center, [0.0, 0.0, 0.0]);
    assert!((r - 1.0).abs() < 1e-6);
    assert!(fit_vertices(&[]).is_empty());
}

#[test]
fn fit_flat_mesh_one_zero_axis_does_not_panic() {
    // A flat quad in the XY plane (zero extent on Z) — non-degenerate
    // overall (X/Y extents are non-zero) so r comes from the max axis.
    let verts = vec![
        [-2.0, -2.0, 5.0],
        [2.0, -2.0, 5.0],
        [-2.0, 2.0, 5.0],
        [2.0, 2.0, 5.0],
    ];
    let (_c, r) = mesh_bounds(&verts);
    assert!((r - 2.0).abs() < 1e-6); // max axis extent 4.0 -> r 2.0
    for v in fit_vertices(&verts) {
        assert!(within_bounds(v), "flat mesh vertex {v:?} escaped bounds");
        assert!(v.iter().all(|c| c.is_finite()));
    }
}
