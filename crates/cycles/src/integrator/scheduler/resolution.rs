// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from the end of `intern/cycles/integrator/render_scheduler.cpp`
//! (`calculate_resolution_divider_for_resolution`,
//! `calculate_resolution_for_divider`, lines 1301-1326): the arithmetic the
//! scheduler uses to render a moving view smaller.

/// `calculate_resolution_divider_for_resolution`: how many times to halve a
/// `width` by `height` image until its area is at most `resolution` squared.
pub fn divider_for_resolution(width: i32, height: i32, resolution: i32) -> i32 {
    let limit = i64::from(resolution) * i64::from(resolution);
    let (mut width, mut height, mut divider) = (width, height, 1);
    while resolution != i32::MAX && i64::from(width) * i64::from(height) > limit {
        width = (width / 2).max(1);
        height = (height / 2).max(1);
        divider <<= 1;
    }
    divider
}

/// `calculate_resolution_for_divider`: the side of the square with the
/// image's area, divided.
pub fn resolution_for_divider(width: i32, height: i32, divider: i32) -> i32 {
    let side = f64::from(width * height).sqrt().round() as i32;
    side / divider
}
