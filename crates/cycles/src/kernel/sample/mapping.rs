// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/sample/mapping.h` (lines 21-100) and
//! `util/math_float3.h` (`make_orthonormals`, line 733): the concentric
//! disk and the cosine-weighted hemisphere.
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use crate::util::vector::{cross, normalize};

/// `polar_to_cartesian` (`util/projection.h:30`).
pub fn polar_to_cartesian(r: f32, phi: f32) -> [f32; 2] {
    [r * phi.cos(), r * phi.sin()]
}

/// `sample_uniform_disk`: Shirley and Chiu's concentric mapping.
pub fn sample_uniform_disk(rand: [f32; 2]) -> [f32; 2] {
    let a = 2.0 * rand[0] - 1.0;
    let b = 2.0 * rand[1] - 1.0;
    if a == 0.0 && b == 0.0 {
        return [0.0, 0.0];
    }
    let (r, phi) = match a * a > b * b {
        true => (a, FRAC_PI_4 * (b / a)),
        false => (b, FRAC_PI_2 - FRAC_PI_4 * (a / b)),
    };
    polar_to_cartesian(r, phi)
}

/// `make_orthonormals`: two vectors that with `n` make an orthonormal frame.
pub fn make_orthonormals(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let a = match n[0] != n[1] || n[0] != n[2] {
        // (1,1,1) x N
        true => [n[2] - n[1], n[0] - n[2], n[1] - n[0]],
        // (-1,1,1) x N
        false => [n[2] - n[1], n[0] + n[2], -n[1] - n[0]],
    };
    let a = normalize(a);
    (a, cross(n, a))
}

/// `sample_cos_hemisphere`: a direction about `n` with density `cos/pi`,
/// and that density.
pub fn sample_cos_hemisphere(n: [f32; 3], rand: [f32; 2]) -> ([f32; 3], f32) {
    let disk = sample_uniform_disk(rand);
    let cos_theta = (1.0 - (disk[0] * disk[0] + disk[1] * disk[1])).max(0.0).sqrt();
    let (t, b) = make_orthonormals(n);
    let wo = [
        disk[0] * t[0] + disk[1] * b[0] + cos_theta * n[0],
        disk[0] * t[1] + disk[1] * b[1] + cos_theta * n[1],
        disk[0] * t[2] + disk[1] * b[2] + cos_theta * n[2],
    ];
    (wo, cos_theta * std::f32::consts::FRAC_1_PI)
}
