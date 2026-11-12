// SPDX-License-Identifier: Apache-2.0
use super::super::near;
use crate::kernel::sample::*;
use crate::util::vector::{dot, len};
#[test]
fn the_concentric_disk_keeps_the_centre_and_stays_in_the_circle() {
    assert_eq!(sample_uniform_disk([0.5, 0.5]), [0.0, 0.0]);
    for i in 0..32 {
        for j in 0..32 {
            let p = sample_uniform_disk([(i as f32 + 0.5) / 32.0, (j as f32 + 0.5) / 32.0]);
            assert!(len(p) <= 1.0 + 1e-6);
        }
    }
    // The corner of the square goes to a point of the circle's edge.
    let p = sample_uniform_disk([1.0, 1.0]);
    assert!((len(p) - 1.0).abs() < 1e-6 && p[0] > 0.0 && p[1] > 0.0);
}
#[test]
fn the_square_of_random_numbers_maps_onto_the_disk_ring_by_ring() {
    // The middle of each side of the square is on the circle.
    assert!(near(&sample_uniform_disk([1.0, 0.5]), &[1.0, 0.0]) && near(&sample_uniform_disk([0.0, 0.5]), &[-1.0, 0.0]));
    assert!(near(&sample_uniform_disk([0.5, 1.0]), &[0.0, 1.0]) && near(&sample_uniform_disk([0.5, 0.0]), &[0.0, -1.0]));
    // A ring of the square is a ring of the disk.
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
#[test]
fn orthonormals_are_at_right_angles_to_the_normal_and_each_other() {
    for n in [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.577_350_26; 3], [-0.6, 0.0, 0.8], [0.0, -1.0, 0.0]] {
        let (a, b) = make_orthonormals(n);
        assert!(dot(a, n).abs() < 1e-6 && dot(b, n).abs() < 1e-6 && dot(a, b).abs() < 1e-6, "{n:?}");
        assert!((len(a) - 1.0).abs() < 1e-5 && (len(b) - 1.0).abs() < 1e-5);
    }
}
#[test]
fn the_cosine_hemisphere_has_the_density_cos_over_pi_about_the_normal() {
    let n = [0.0, 0.6, 0.8];
    let (mut mean_cos, count) = (0.0f64, 64 * 64);
    for i in 0..64 {
        for j in 0..64 {
            let (wo, pdf) = sample_cos_hemisphere(n, [(i as f32 + 0.5) / 64.0, (j as f32 + 0.5) / 64.0]);
            let cos = dot(wo, n);
            assert!((len(wo) - 1.0).abs() < 1e-5 && cos >= -1e-6);
            assert!((pdf - cos.max(0.0) / std::f32::consts::PI).abs() < 1e-5);
            mean_cos += f64::from(cos);
        }
    }
    // The mean of cos under a cosine density is 2/3.
    assert!((mean_cos / f64::from(count) - 2.0 / 3.0).abs() < 0.01);
}
