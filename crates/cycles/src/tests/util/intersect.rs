// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

const A: [f32; 3] = [0.0, 0.0, 0.0];
const B: [f32; 3] = [4.0, 0.0, 0.0];
const C: [f32; 3] = [0.0, 4.0, 0.0];

/// A ray along z from two units in front of the plane of the triangle.
fn down(x: f32, y: f32) -> Option<[f32; 3]> {
    ray_triangle_intersect([x, y, -2.0], [0.0, 0.0, 1.0], 0.0, 10.0, A, B, C)
}

fn near(left: [f32; 3], right: [f32; 3]) -> bool {
    left.iter().zip(right).all(|(left, right)| (left - right).abs() < 1e-5)
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
fn outside_the_triangle_or_the_ray_s_reach_there_is_no_hit() {
    assert!(down(3.0, 3.0).is_none() && down(-0.5, 1.0).is_none() && down(1.0, -0.5).is_none());
    let reach = |tmin: f32, tmax: f32| ray_triangle_intersect([1.0, 1.0, -2.0], [0.0, 0.0, 1.0], tmin, tmax, A, B, C);
    assert!(reach(0.0, 1.9).is_none() && reach(2.1, 10.0).is_none() && reach(2.0, 2.0).is_some());
    // Behind the ray's start.
    assert!(ray_triangle_intersect([1.0, 1.0, 2.0], [0.0, 0.0, 1.0], 0.0, 10.0, A, B, C).is_none());
}

#[test]
fn a_triangle_is_hit_from_either_side_and_not_along_its_plane() {
    let back = ray_triangle_intersect([1.0, 2.0, 3.0], [0.0, 0.0, -1.0], 0.0, 10.0, A, B, C);
    assert!(near(back.expect("from behind"), [0.25, 0.5, 3.0]));
    assert!(ray_triangle_intersect([-1.0, 1.0, 0.0], [1.0, 0.0, 0.0], 0.0, 10.0, A, B, C).is_none());
}

#[test]
fn two_triangles_that_share_an_edge_leave_no_gap_along_it() {
    // The square of the diagonal from A to D, cut along the other one.
    let d = [4.0, 4.0, 0.0];
    for step in 0..=40 {
        let x = step as f32 * 0.1;
        let hit = |b: [f32; 3], c: [f32; 3]| ray_triangle_intersect([x, 4.0 - x, -1.0], [0.0, 0.0, 1.0], 0.0, 10.0, b, c, d).is_some();
        assert!(down(x, 4.0 - x).is_some() || hit(B, C) || hit(C, B), "a ray through the shared edge at x = {x}");
    }
}

#[test]
fn a_slanted_ray_s_distance_is_along_the_ray() {
    let d = vector::normalize([3.0, 0.0, 4.0]);
    let hit = ray_triangle_intersect([-2.0, 1.0, -4.0], d, 0.0, 10.0, A, B, C).expect("through 1, 1");
    assert!(near(hit, [0.25, 0.25, 5.0]), "{hit:?}");
}
