//! Property 5.5 — Viewport-fit bounds.
//!
//! Feature: blend-file-mesh-loading, Property 5.5: viewport-fit bounds
//! Validates: Requirement 4.6
//!
//! For any non-empty vertex set, the viewport-fit transform produces only
//! finite results, every fitted vertex sits within [-MESH_SCALE, MESH_SCALE]
//! on each axis, and degenerate inputs (single-point mesh) never panic.

use phiber::v2::scene::{FIT_EPSILON, MESH_SCALE, fit_vertex, fit_vertices, mesh_bounds};
use proptest::prelude::*;

/// Strategy: a non-empty Vec of finite [f32; 3] vertices in a bounded range.
/// We avoid NaN/Inf by generating from a closed interval.
fn arb_nonempty_vertices() -> impl Strategy<Value = Vec<[f32; 3]>> {
    prop::collection::vec(
        [
            -1000.0f32..=1000.0,
            -1000.0f32..=1000.0,
            -1000.0f32..=1000.0,
        ]
        .prop_map(|[x, y, z]| [x, y, z]),
        1..=64,
    )
}

/// Assert every component of `v` is in `[-MESH_SCALE - eps, MESH_SCALE + eps]`.
fn within_fit_bounds(v: [f32; 3]) -> bool {
    let eps = 1e-4_f32;
    v.iter().all(|&c| c >= -MESH_SCALE - eps && c <= MESH_SCALE + eps)
}

proptest! {
    // Feature: blend-file-mesh-loading, Property 5.5: viewport-fit bounds

    /// Non-empty vertex set: every fitted vertex is finite and within bounds.
    #[test]
    fn fit_vertices_all_within_mesh_scale(verts in arb_nonempty_vertices()) {
        let fitted = fit_vertices(&verts);
        prop_assert_eq!(fitted.len(), verts.len());
        for v in &fitted {
            prop_assert!(
                v.iter().all(|c| c.is_finite()),
                "fitted vertex has non-finite component: {v:?}"
            );
            prop_assert!(
                within_fit_bounds(*v),
                "fitted vertex {v:?} out of [-MESH_SCALE, MESH_SCALE] bounds"
            );
        }
    }

    /// mesh_bounds always returns a finite, positive r for non-empty input.
    #[test]
    fn mesh_bounds_r_is_finite_and_positive(verts in arb_nonempty_vertices()) {
        let (_center, r) = mesh_bounds(&verts);
        prop_assert!(r.is_finite(), "r is not finite: {r}");
        prop_assert!(r > 0.0, "r must be positive, got {r}");
    }

    /// mesh_bounds center is finite on non-empty input.
    #[test]
    fn mesh_bounds_center_is_finite(verts in arb_nonempty_vertices()) {
        let (center, _r) = mesh_bounds(&verts);
        for c in center {
            prop_assert!(c.is_finite(), "center component non-finite: {c}");
        }
    }

    /// fit_vertex with r = 1 (degenerate fallback) produces only finite outputs.
    #[test]
    fn fit_vertex_degenerate_r_is_finite(
        x in -1000.0f32..1000.0,
        y in -1000.0f32..1000.0,
        z in -1000.0f32..1000.0,
    ) {
        let v = [x, y, z];
        let center = [x, y, z]; // same as vertex -> maps to origin
        let result = fit_vertex(v, center, 1.0);
        for c in result {
            prop_assert!(c.is_finite(), "degenerate fit_vertex non-finite: {c}");
        }
    }
}

// --- Deterministic unit cases ---

#[test]
fn empty_vertices_returns_empty() {
    assert!(fit_vertices(&[]).is_empty());
}

#[test]
fn single_point_maps_to_origin() {
    // A single-point cloud: every vertex identical -> AABB extent 0 -> r guard = 1.0.
    let verts = vec![[5.0, -3.0, 7.0]; 8];
    let fitted = fit_vertices(&verts);
    for v in fitted {
        assert!(
            v.iter().all(|c| c.is_finite()),
            "single-point vertex not finite"
        );
        // All coincident points recentre to origin, then / r * MESH_SCALE = 0.
        for c in v {
            assert!(c.abs() < 1e-5, "expected ~0.0, got {c}");
        }
    }
}

#[test]
fn unit_cube_fits_within_mesh_scale() {
    let corners: Vec<[f32; 3]> = [-1.0_f32, 1.0]
        .iter()
        .flat_map(|&x| {
            [-1.0_f32, 1.0].iter().flat_map(move |&y| {
                [-1.0_f32, 1.0].iter().map(move |&z| [x, y, z])
            })
        })
        .collect();
    let fitted = fit_vertices(&corners);
    for v in &fitted {
        assert!(
            within_fit_bounds(*v),
            "unit cube vertex {v:?} escaped bounds"
        );
    }
}

#[test]
fn fit_epsilon_constant_is_small_positive() {
    assert!(FIT_EPSILON > 0.0);
    assert!(FIT_EPSILON < 1e-3);
}
