// SPDX-License-Identifier: Apache-2.0
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
