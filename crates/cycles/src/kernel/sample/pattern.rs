// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/sample/pattern.h` (lines 31-175):
//! `path_rng_1D` to `path_rng_3D` and `path_rng_pixel_init`, for all the
//! patterns: Sobol-Burley and the blue-noise ones (`blue_noise_indexing`),
//! and the tabulated Sobol one (`tabulated.rs`). Blender 5.2's default,
//! `AUTOMATIC`, is `BLUE_NOISE_PURE` when rendering in the background
//! (`blender/sync.cpp:435-459`); a scene file that holds a pattern says
//! `sampling_pattern` 1, the Classic one, which is the tabulated Sobol.
use super::tabulated::{tabulated_sobol_sample_1d, tabulated_sobol_sample_2d, tabulated_sobol_sample_3d};
use super::sobol::{sobol_burley_sample_1d, sobol_burley_sample_2d, sobol_burley_sample_3d};
use super::util::{morton2d, nested_uniform_scramble_base4};
use crate::kernel::constant;
use crate::util::hash_iqnt2d;

/// `SamplingPattern`, by Cycles' number (`data/constant.tsv`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingPattern {
    SobolBurley,
    TabulatedSobol,
    BlueNoisePure,
    BlueNoiseFirst,
}

impl SamplingPattern {
    pub fn of(number: i32) -> SamplingPattern {
        let is = |name: &str| constant(name) == Some(i64::from(number));
        match () {
            _ if is("SAMPLING_PATTERN_BLUE_NOISE_PURE") => Self::BlueNoisePure,
            _ if is("SAMPLING_PATTERN_BLUE_NOISE_FIRST") => Self::BlueNoiseFirst,
            _ if is("SAMPLING_PATTERN_TABULATED_SOBOL") => Self::TabulatedSobol,
            _ => Self::SobolBurley,
        }
    }
}

/// What `kernel_data.integrator` holds for sampling: the pattern, the seed
/// and the two numbers the pattern indexes with
/// (`blue_noise_sequence_length`, `sobol_index_mask`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SamplerData {
    pub pattern: SamplingPattern,
    pub seed: u32,
    pub sequence_length: u32,
    pub index_mask: u32,
    /// `tabulated_sobol_sequence_size`: the points of each pattern of the table.
    pub sequence_size: u32,
    pub scrambling_distance: f32,
}

/// `blue_noise_indexing`: the index, seed and mask Sobol-Burley is called
/// with for a pixel's sample.
fn indexing(s: &SamplerData, pixel_index: u32, sample: u32) -> (u32, u32, u32) {
    let section = pixel_index.wrapping_mul(s.sequence_length);
    match s.pattern {
        // One sequence per pixel, with the length mask optimisation.
        SamplingPattern::SobolBurley | SamplingPattern::TabulatedSobol => (sample, pixel_index, s.index_mask),
        // A single sequence (seed 0) of which each pixel has a section.
        SamplingPattern::BlueNoisePure => (sample.wrapping_add(section), 0, 0xffff_ffff),
        // A 1 SPP sequence for the first sample, another for the rest.
        SamplingPattern::BlueNoiseFirst => match sample {
            0 => (pixel_index, 0x0cd0_519f, 0xffff_ffff),
            _ => ((sample - 1).wrapping_add(section), 0, 0xffff_ffff),
        },
    }
}

/// `path_rng_1D`.
pub fn path_rng_1d(s: &SamplerData, lut: &[f32], rng_pixel: u32, sample: u32, dimension: u32) -> f32 {
    if s.pattern == SamplingPattern::TabulatedSobol {
        return tabulated_sobol_sample_1d(s, lut, sample, rng_pixel, dimension);
    }
    let (index, seed, mask) = indexing(s, rng_pixel, sample);
    sobol_burley_sample_1d(index, dimension, seed, mask)
}

/// `path_rng_2D`.
pub fn path_rng_2d(s: &SamplerData, lut: &[f32], rng_pixel: u32, sample: u32, dimension: u32) -> [f32; 2] {
    if s.pattern == SamplingPattern::TabulatedSobol {
        return tabulated_sobol_sample_2d(s, lut, sample, rng_pixel, dimension);
    }
    let (index, seed, mask) = indexing(s, rng_pixel, sample);
    sobol_burley_sample_2d(index, dimension, seed, mask)
}

/// `path_rng_3D`.
pub fn path_rng_3d(s: &SamplerData, lut: &[f32], rng_pixel: u32, sample: u32, dimension: u32) -> [f32; 3] {
    if s.pattern == SamplingPattern::TabulatedSobol {
        return tabulated_sobol_sample_3d(s, lut, sample, rng_pixel, dimension);
    }
    let (index, seed, mask) = indexing(s, rng_pixel, sample);
    sobol_burley_sample_3d(index, dimension, seed, mask)
}

/// `path_rng_pixel_init`: the white-noise samplers hash the pixel, the
/// blue-noise ones place it on a base-4 scrambled Morton curve.
pub fn path_rng_pixel_init(s: &SamplerData, x: u32, y: u32) -> u32 {
    match s.pattern {
        SamplingPattern::TabulatedSobol | SamplingPattern::SobolBurley => hash_iqnt2d(x, y) ^ s.seed,
        _ => nested_uniform_scramble_base4(morton2d(x, y), s.seed),
    }
}

#[cfg(test)]
mod test;
