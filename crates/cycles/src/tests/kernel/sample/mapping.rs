// SPDX-License-Identifier: Apache-2.0
use super::super::near;
use crate::kernel::sample::*;

#[test]
fn the_square_of_random_numbers_maps_onto_the_disk_ring_by_ring() {
    assert_eq!(sample_uniform_disk([0.5, 0.5]), [0.0; 2]);
    // The middle of each side of the square is on the circle.
    assert!(near(&sample_uniform_disk([1.0, 0.5]), &[1.0, 0.0]) && near(&sample_uniform_disk([0.0, 0.5]), &[-1.0, 0.0]));
    assert!(near(&sample_uniform_disk([0.5, 1.0]), &[0.0, 1.0]) && near(&sample_uniform_disk([0.5, 0.0]), &[0.0, -1.0]));
    // And so are its corners: a ring of the square is a ring of the disk.
    let corner = sample_uniform_disk([1.0, 1.0]);
    assert!(near(&[corner[0] * corner[0] + corner[1] * corner[1]], &[1.0]));
    let inner = sample_uniform_disk([0.75, 0.6]);
    assert!(near(&[(inner[0] * inner[0] + inner[1] * inner[1]).sqrt()], &[0.5]));
}

#[test]
fn a_polygon_s_corners_are_on_the_circle_and_its_points_within() {
    let corner = std::f32::consts::FRAC_1_SQRT_2;
    assert!(near(&regular_polygon_sample(4.0, 0.0, [0.0, 0.0]), &[corner, corner]));
    assert!(near(&regular_polygon_sample(4.0, 0.0, [0.25, 0.0]), &[-corner, corner]));
    // Turned by an eighth, that corner is on the axis.
    assert!(near(&regular_polygon_sample(4.0, std::f32::consts::FRAC_PI_4, [0.0, 0.0]), &[0.0, 1.0]));
    // Where the first number's share of a corner runs out, the second
    // number goes from the middle to the next corner.
    assert!(near(&regular_polygon_sample(5.0, 0.3, [0.199_999, 0.0]), &[0.0, 0.0]));
    let next = regular_polygon_sample(4.0, 0.0, [0.249_999, 1.0]);
    assert!(near(&next, &[corner, -corner]), "{next:?}");
    for step in 0..50 {
        let point = regular_polygon_sample(6.0, 0.2, [step as f32 / 50.0, (step * 7 % 50) as f32 / 50.0]);
        assert!(point[0] * point[0] + point[1] * point[1] <= 1.0 + 1e-5, "{point:?}");
    }
}
