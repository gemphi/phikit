// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/math_cdf.h` and `math_cdf.cpp`
//! (`util_cdf_evaluate`, `util_cdf_invert`, `util_cdf_inverted`): a table
//! to draw samples from a function by, such as a pixel filter.

/// `util_cdf_evaluate`: the running sum of `functor` over `from..to` at
/// `resolution` steps, scaled to end at one. `resolution + 1` numbers.
pub fn cdf_evaluate(resolution: usize, from: f32, to: f32, functor: impl Fn(f32) -> f32) -> Vec<f32> {
    let range = to - from;
    let mut cdf = vec![0.0f32; resolution + 1];
    for index in 0..resolution {
        let x = from + range * index as f32 / (resolution as f32 - 1.0);
        cdf[index + 1] = cdf[index] + functor(x).abs();
    }
    let scale = match cdf[resolution] == 0.0 {
        true => 0.0,
        false => 1.0 / cdf[resolution],
    };
    cdf.iter_mut().for_each(|value| *value *= scale);
    cdf[resolution] = 1.0;
    cdf
}

/// Where `x` falls in `cdf`: the entry and how far toward the next one.
/// `skip` is how many entries not above `x` to step back over; past the
/// last entry the answer is `end`, with nothing toward a next one.
fn locate(cdf: &[f32], x: f32, skip: usize, end: usize) -> (usize, f32) {
    let index = cdf.partition_point(|value| *value <= x) - skip;
    match index < cdf.len() - 1 {
        true => (index, (x - cdf[index]) / (cdf[index + 1] - cdf[index])),
        false => (end, 0.0),
    }
}

/// `util_cdf_invert`: for `resolution` evenly spaced random numbers, the
/// place in `from..to` each one draws. `make_symmetric` mirrors a half
/// range about its middle, as a pixel filter is; its last entry is left
/// at zero, as Cycles leaves it.
pub fn cdf_invert(resolution: usize, from: f32, to: f32, cdf: &[f32], make_symmetric: bool) -> Vec<f32> {
    let range = to - from;
    let mut inv_cdf = vec![0.0f32; resolution];
    match make_symmetric {
        true => {
            let half_size = (resolution - 1) / 2;
            for index in 0..=half_size {
                let x = index as f32 / half_size as f32;
                let (entry, toward) = locate(cdf, x, 0, cdf.len() - 1);
                let y = ((entry as f32 + toward) / (resolution as f32 - 1.0)) * (2.0 * range);
                inv_cdf[half_size + index] = 0.5 * (1.0 + y);
                inv_cdf[half_size - index] = 0.5 * (1.0 - y);
            }
        }
        false => {
            let inv_resolution = 1.0 / resolution as f32;
            for (index, slot) in inv_cdf.iter_mut().enumerate() {
                let x = (index as f32 + 0.5) * inv_resolution;
                let (entry, toward) = locate(cdf, x, 1, resolution);
                *slot = from + range * (entry as f32 + toward) * inv_resolution;
            }
        }
    }
    inv_cdf
}

/// `util_cdf_inverted`: the two above in one. The sum is taken at one step
/// fewer than the table has, to match the pixel filter table of old.
pub fn cdf_inverted(resolution: usize, from: f32, to: f32, functor: impl Fn(f32) -> f32, make_symmetric: bool) -> Vec<f32> {
    let cdf = cdf_evaluate(resolution - 1, from, to, functor);
    cdf_invert(resolution, from, to, &cdf, make_symmetric)
}
