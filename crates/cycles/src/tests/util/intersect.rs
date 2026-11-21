// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::util::{ray_triangle_intersect, ray_triangle_intersect_self, vector};
const A: [f32; 3] = [0.0, 0.0, 0.0];
const B: [f32; 3] = [1.0, 0.0, 0.0];
const C: [f32; 3] = [0.0, 1.0, 0.0];
/// The triangle four times as large, for the tests that walk its edges.
const BIG: [[f32; 3]; 3] = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]];
/// A hit as `u`, `v` and `t`, against the large triangle.
fn big(p: [f32; 3], d: [f32; 3], tmin: f32, tmax: f32) -> Option<[f32; 3]> {
    ray_triangle_intersect(p, d, tmin, tmax, BIG[0], BIG[1], BIG[2]).map(|hit| [hit.u, hit.v, hit.t])
}
/// A ray along z from two units in front of the plane of the triangle.
fn down(x: f32, y: f32) -> Option<[f32; 3]> {
    big([x, y, -2.0], [0.0, 0.0, 1.0], 0.0, 10.0)
}
fn near(left: [f32; 3], right: [f32; 3]) -> bool {
    left.iter().zip(right).all(|(left, right)| (left - right).abs() < 1e-5)
}
#[test]
fn a_ray_through_the_triangle_hits_it_at_the_closed_form_distance_and_coordinates() {
    let hit = ray_triangle_intersect([0.25, 0.5, 2.0], [0.0, 0.0, -1.0], 0.0, 100.0, A, B, C).expect("a hit");
    assert!((hit.t - 2.0).abs() < 1e-6);
    // P = A + u (B - A) + v (C - A).
    assert!((hit.u - 0.25).abs() < 1e-6 && (hit.v - 0.5).abs() < 1e-6, "{hit:?}");
}
#[test]
fn a_hit_gives_the_weights_of_the_second_and_third_corner_and_the_distance() {
    assert!(near(down(1.0, 2.0).expect("inside"), [0.25, 0.5, 2.0]));
    // At a corner all the weight is that corner's.
    assert!(near(down(4.0, 0.0).expect("the second corner"), [1.0, 0.0, 2.0]));
    assert!(near(down(0.0, 4.0).expect("the third corner"), [0.0, 1.0, 2.0]));
    assert!(near(down(0.0, 0.0).expect("the first corner"), [0.0, 0.0, 2.0]));
}
#[test]
fn a_slanted_ray_has_the_distance_along_the_ray() {
    // From (0.25, 0.25, 1) towards (0.25, 0.25, 0) by a direction that is not unit length.
    let hit = ray_triangle_intersect([0.0, 0.0, 1.0], [0.25, 0.25, -1.0], 0.0, 100.0, A, B, C).expect("a hit");
    assert!((hit.t - 1.0).abs() < 1e-6 && (hit.u - 0.25).abs() < 1e-6);
    // And of length one.
    let d = vector::normalize([3.0, 0.0, 4.0]);
    let hit = big([-2.0, 1.0, -4.0], d, 0.0, 10.0).expect("through 1, 1");
    assert!(near(hit, [0.25, 0.25, 5.0]), "{hit:?}");
}
#[test]
fn a_ray_beside_behind_or_short_of_the_triangle_misses() {
    let down_z = [0.0, 0.0, -1.0];
    assert!(ray_triangle_intersect([0.8, 0.8, 2.0], down_z, 0.0, 100.0, A, B, C).is_none());
    assert!(ray_triangle_intersect([0.25, 0.25, 2.0], [0.0, 0.0, 1.0], 0.0, 100.0, A, B, C).is_none());
    assert!(ray_triangle_intersect([0.25, 0.25, 2.0], down_z, 0.0, 1.5, A, B, C).is_none());
    assert!(ray_triangle_intersect([0.25, 0.25, 2.0], down_z, 2.5, 100.0, A, B, C).is_none());
    // Parallel to the plane.
    assert!(ray_triangle_intersect([0.25, 0.25, 2.0], [1.0, 0.0, 0.0], 0.0, 100.0, A, B, C).is_none());
    // Outside the edges, and at the ends of the ray's reach.
    assert!(down(3.0, 3.0).is_none() && down(-0.5, 1.0).is_none() && down(1.0, -0.5).is_none());
    let reach = |tmin: f32, tmax: f32| big([1.0, 1.0, -2.0], [0.0, 0.0, 1.0], tmin, tmax);
    assert!(reach(0.0, 1.9).is_none() && reach(2.1, 10.0).is_none() && reach(2.0, 2.0).is_some());
}
#[test]
fn the_triangle_is_hit_from_either_side() {
    assert!(ray_triangle_intersect([0.25, 0.25, 2.0], [0.0, 0.0, -1.0], 0.0, 100.0, A, B, C).is_some());
    assert!(ray_triangle_intersect([0.25, 0.25, -2.0], [0.0, 0.0, 1.0], 0.0, 100.0, A, B, C).is_some());
    let back = big([1.0, 2.0, 3.0], [0.0, 0.0, -1.0], 0.0, 10.0);
    assert!(near(back.expect("from behind"), [0.25, 0.5, 3.0]));
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
    // The other diagonal of the large square, shared by the large
    // triangle and the one beyond it.
    let far = [4.0, 4.0, 0.0];
    for step in 0..=40 {
        let x = step as f32 * 0.1;
        let hit = |b: [f32; 3], c: [f32; 3]| ray_triangle_intersect([x, 4.0 - x, -1.0], [0.0, 0.0, 1.0], 0.0, 10.0, b, c, far).is_some();
        assert!(down(x, 4.0 - x).is_some() || hit(BIG[1], BIG[2]) || hit(BIG[2], BIG[1]), "a ray through the shared edge at x = {x}");
    }
}
#[test]
fn the_self_test_is_true_for_a_ray_that_cannot_reach_a_neighbour() {
    // Leaving the middle of the triangle upward: it is inside, the test says "this triangle".
    assert!(ray_triangle_intersect_self([0.3, 0.3, 0.0], [0.0, 0.0, 1.0], [A, B, C]));
    // Far outside the triangle's edges, the test says it would miss.
    assert!(!ray_triangle_intersect_self([2.0, 2.0, 0.0], [0.0, 0.0, 1.0], [A, B, C]));
}
