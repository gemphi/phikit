// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/sample/tabulated_sobol.h` (lines
//! 10-178): the sampler of the Classic pattern. A pixel's samples are read
//! from the table of 256 patterns (`scene/integrator/tabulated.rs`), which
//! pattern and which point in it shuffled by the pixel's hash and the
//! dimension. With a scrambling distance under one every pixel reads from
//! the same sequence, moved a little by a Cranley-Patterson rotation.
use super::pattern::SamplerData;
use super::util::reversed_bit_owen;
use crate::kernel::constant;
use crate::util::{hash_shuffle_uint, hash_wang_seeded_float, hash_wang_seeded_uint};

/// `NUM_TAB_SOBOL_PATTERNS` and `NUM_TAB_SOBOL_DIMENSIONS`.
fn patterns() -> u32 {
    constant("NUM_TAB_SOBOL_PATTERNS").unwrap_or(256) as u32
}

const DIMENSIONS: usize = 4;

/// `nested_uniform_scramble`: base-2 Owen scrambling of an integer.
fn nested_uniform_scramble(i: u32, seed: u32) -> u32 {
    reversed_bit_owen(i.reverse_bits(), seed).reverse_bits()
}

/// `tabulated_sobol_shuffled_sample_index`: the place in the table of the
/// sample, within the pattern the dimension and seed pick.
fn shuffled_sample_index(s: &SamplerData, sample: u32, dimension: u32, seed: u32) -> usize {
    let sample_count = s.sequence_size;
    // Shuffle the pattern order and the sample index to decorrelate the
    // dimensions; the sample is shuffled only within its sequence.
    let pattern = hash_shuffle_uint(dimension, patterns(), seed);
    // `sample_count` is a power of two, so this is a mask.
    let mask = sample_count.wrapping_sub(1);
    let shuffled = nested_uniform_scramble(sample, hash_wang_seeded_uint(dimension, seed));
    let sample = (sample & !mask) | (shuffled & mask);
    ((u64::from(pattern) * u64::from(sample_count) + u64::from(sample)) % (u64::from(sample_count) * u64::from(patterns()))) as usize
}

/// The `n` floats of a sample (`n` up to four) and the rotation that moves
/// them when pixels share a sequence.
fn tabulated_sample<const N: usize>(s: &SamplerData, lut: &[f32], sample: u32, rng_hash: u32, dimension: u32) -> [f32; N] {
    let rotate = s.scrambling_distance < 1.0;
    let seed = if rotate { s.seed } else { rng_hash };
    let index = shuffled_sample_index(s, sample, dimension, seed) * DIMENSIONS;
    // The extra seeds of the 2nd, 3rd and 4th dimension's rotation.
    const SALT: [u32; 4] = [0, 0xca0e_1151, 0xbf60_4c5a, 0x9963_4d1d];
    std::array::from_fn(|axis| {
        let x = lut.get(index + axis).copied().unwrap_or(0.0);
        match rotate {
            true => {
                let jitter = hash_wang_seeded_float(dimension, rng_hash ^ SALT[axis]) * s.scrambling_distance;
                let moved = x + jitter;
                moved - moved.floor()
            }
            false => x,
        }
    })
}

/// `tabulated_sobol_sample_1D`.
pub fn tabulated_sobol_sample_1d(s: &SamplerData, lut: &[f32], sample: u32, rng_hash: u32, dimension: u32) -> f32 {
    tabulated_sample::<1>(s, lut, sample, rng_hash, dimension)[0]
}

/// `tabulated_sobol_sample_2D`.
pub fn tabulated_sobol_sample_2d(s: &SamplerData, lut: &[f32], sample: u32, rng_hash: u32, dimension: u32) -> [f32; 2] {
    tabulated_sample::<2>(s, lut, sample, rng_hash, dimension)
}

/// `tabulated_sobol_sample_3D`.
pub fn tabulated_sobol_sample_3d(s: &SamplerData, lut: &[f32], sample: u32, rng_hash: u32, dimension: u32) -> [f32; 3] {
    tabulated_sample::<3>(s, lut, sample, rng_hash, dimension)
}

#[cfg(test)]
mod test;
