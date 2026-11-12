// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/sample/mapping.h`
//! (`sample_uniform_disk`, lines 19-42; `regular_polygon_sample`, lines
//! 187-213): two random numbers from 0 to 1 made into a point of a shape.
//! The hemisphere, cone and sphere come with the closures and lights that
//! sample them.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use crate::util::polar_to_cartesian;

/// `sample_uniform_disk`: a point of the disk of radius one, by the
/// concentric mapping, which keeps points that were apart apart.
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
