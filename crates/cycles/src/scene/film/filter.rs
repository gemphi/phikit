// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/film.cpp` (`filter_func_box`,
//! `filter_func_gaussian`, `filter_func_blackman_harris`, `filter_table`,
//! lines 24-84): the pixel filter, as a table the camera kernel draws the
//! place of a sample inside its pixel from.

use crate::kernel::constant;
use crate::util::cdf_inverted;

/// `filter_func_gaussian`.
fn gaussian(v: f32, width: f32) -> f32 {
    let v = v * 6.0 / width;
    (-2.0 * v * v).exp()
}

/// `filter_func_blackman_harris`.
fn blackman_harris(v: f32, width: f32) -> f32 {
    let v = std::f32::consts::TAU * (v / width + 0.5);
    0.35875 - 0.48829 * v.cos() + 0.14128 * (2.0 * v).cos() - 0.01168 * (3.0 * v).cos()
}

/// `filter_table`: the table for the filter of the film's `filter_enum`
/// key (`box`, `gaussian` or `blackman_harris`) and width in pixels.
/// `FILTER_TABLE_SIZE` numbers; an unknown key is the box.
pub fn filter_table(filter: &str, width: f32) -> Vec<f32> {
    let size = constant("FILTER_TABLE_SIZE").unwrap_or(0) as usize;
    // The wider filters reach past their width.
    let (function, width): (fn(f32, f32) -> f32, f32) = match filter {
        "gaussian" => (gaussian, width * 3.0),
        "blackman_harris" => (blackman_harris, width * 2.0),
        _ => (|_, _| 1.0, width),
    };
    cdf_inverted(size, 0.0, width * 0.5, |x| function(x, width), true)
}
