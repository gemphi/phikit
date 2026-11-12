// SPDX-License-Identifier: Apache-2.0
use crate::util::{ray_triangle_intersect, ray_triangle_intersect_self};

const A: [f32; 3] = [0.0, 0.0, 0.0];
const B: [f32; 3] = [1.0, 0.0, 0.0];
const C: [f32; 3] = [0.0, 1.0, 0.0];

#[test]
fn a_ray_through_the_triangle_hits_it_at_the_closed_form_distance_and_coordinates() {
    let hit = ray_triangle_intersect([0.25, 0.5, 2.0], [0.0, 0.0, -1.0], 0.0, 100.0, A, B, C).expect("a hit");
    assert!((hit.t - 2.0).abs() < 1e-6);
    // P = A + u (B - A) + v (C - A).
    assert!((hit.u - 0.25).abs() < 1e-6 && (hit.v - 0.5).abs() < 1e-6, "{hit:?}");
}

#[test]
fn a_slanted_ray_has_the_distance_along_the_ray() {
    // From (0.25, 0.25, 1) towards (0.25, 0.25, 0) by a direction that is not unit length.
    let hit = ray_triangle_intersect([0.0, 0.0, 1.0], [0.25, 0.25, -1.0], 0.0, 100.0, A, B, C).expect("a hit");
    assert!((hit.t - 1.0).abs() < 1e-6 && (hit.u - 0.25).abs() < 1e-6);
}

#[test]
fn a_ray_beside_behind_or_short_of_the_triangle_misses() {
    let down = [0.0, 0.0, -1.0];
    assert!(ray_triangle_intersect([0.8, 0.8, 2.0], down, 0.0, 100.0, A, B, C).is_none());
    assert!(ray_triangle_intersect([0.25, 0.25, 2.0], [0.0, 0.0, 1.0], 0.0, 100.0, A, B, C).is_none());
    assert!(ray_triangle_intersect([0.25, 0.25, 2.0], down, 0.0, 1.5, A, B, C).is_none());
    assert!(ray_triangle_intersect([0.25, 0.25, 2.0], down, 2.5, 100.0, A, B, C).is_none());
    // Parallel to the plane.
    assert!(ray_triangle_intersect([0.25, 0.25, 2.0], [1.0, 0.0, 0.0], 0.0, 100.0, A, B, C).is_none());
}

#[test]
fn the_triangle_is_hit_from_either_side() {
    assert!(ray_triangle_intersect([0.25, 0.25, 2.0], [0.0, 0.0, -1.0], 0.0, 100.0, A, B, C).is_some());
    assert!(ray_triangle_intersect([0.25, 0.25, -2.0], [0.0, 0.0, 1.0], 0.0, 100.0, A, B, C).is_some());
}

#[test]
fn the_edge_between_two_triangles_is_hit_by_one_of_them_at_least() {
    // The diagonal of a square: the watertight test hits one or both.
    let (d, up) = ([1.0, 1.0, 0.0], [0.0, 0.0, -1.0]);
    for t in [0.1f32, 0.37, 0.5, 0.81] {
        let p = [t, t, 1.0];
        let first = ray_triangle_intersect(p, up, 0.0, 10.0, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], d).is_some();
        let second = ray_triangle_intersect(p, up, 0.0, 10.0, [0.0, 0.0, 0.0], d, [0.0, 1.0, 0.0]).is_some();
        assert!(first || second, "t = {t}");
    }
}

#[test]
fn the_self_test_is_true_for_a_ray_that_cannot_reach_a_neighbour() {
    // Leaving the middle of the triangle upward: it is inside, the test says "this triangle".
    assert!(ray_triangle_intersect_self([0.3, 0.3, 0.0], [0.0, 0.0, 1.0], [A, B, C]));
    // Far outside the triangle's edges, the test says it would miss.
    assert!(!ray_triangle_intersect_self([2.0, 2.0, 0.0], [0.0, 0.0, 1.0], [A, B, C]));
}
