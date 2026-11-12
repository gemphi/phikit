// SPDX-FileCopyrightText: 2009-2010 Sony Pictures Imageworks Inc., et al. All Rights Reserved.
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/sample/mapping.h`
//! (`sample_uniform_disk`, lines 19-42; `sample_cos_hemisphere`, 89-100;
//! `regular_polygon_sample`, 187-213) and `util/math_float3.h`
//! (`make_orthonormals`, line 733): two random numbers from 0 to 1 made
//! into a point of a shape. The cone and the sphere come with the closures
//! and lights that sample them.
use std::f32::consts::{FRAC_1_PI, FRAC_PI_2, FRAC_PI_4, PI};
use crate::util::polar_to_cartesian;
use crate::util::vector::{cross, normalize};
/// `sample_uniform_disk`: a point of the disk of radius one, by Shirley
/// and Chiu's concentric mapping, which keeps points that were apart apart.
pub fn sample_uniform_disk(rand: [f32; 2]) -> [f32; 2] {
    let (a, b) = (2.0 * rand[0] - 1.0, 2.0 * rand[1] - 1.0);
    if a == 0.0 && b == 0.0 {
        return [0.0; 2];
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
    (wo, cos_theta * FRAC_1_PI)
}
/// `regular_polygon_sample`: a point of the polygon of `corners` corners
/// that fits the circle of radius one, turned by `rotation`.
pub fn regular_polygon_sample(corners: f32, rotation: f32, rand: [f32; 2]) -> [f32; 2] {
    // The first number picks a corner, and what is left of it is used again.
    let corner = (rand[0] * corners).floor();
    let u = rand[0] * corners - corner;
    // Weights of a point of a triangle, evenly spread.
    let u = u.sqrt();
    let v = rand[1] * u;
    let u = 1.0 - u;
    // The point, in the triangle from the middle to one side.
    let angle = PI / corners;
    let point = [(u + v) * angle.cos(), (u - v) * angle.sin()];
    let (sin, cos) = (rotation + corner * 2.0 * angle).sin_cos();
    [cos * point[0] - sin * point[1], sin * point[0] + cos * point[1]]
}

#[cfg(test)]
mod test;
