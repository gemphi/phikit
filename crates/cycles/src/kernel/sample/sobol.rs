// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/sample/sobol_burley.h` (lines 1-203):
//! a shuffled, Owen-scrambled Sobol sampler after Brent Burley's "Practical
//! Hash-based Owen Scrambling" (2020). Higher dimensions are padded: each
//! `dimension` of a call decorrelates by seeding, not by another Sobol
//! dimension. The direction numbers are `data/sobol.tsv`
//! (`tool/sobol.py` reads them from `kernel/tables.h`).
use std::sync::OnceLock;

use super::util::reversed_bit_owen;
use crate::util::{hash_hp_uint, row_list, uint_to_float_excl};

/// `sobol_burley_table`.
fn table() -> &'static [[u32; 32]; 4] {
    static TABLE: OnceLock<[[u32; 32]; 4]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [[0u32; 32]; 4];
        for row in row_list(include_str!("../../../data/sobol.tsv")) {
            let dimension: usize = row[0].parse().expect("data/sobol.tsv: a dimension");
            for (slot, cell) in table[dimension].iter_mut().zip(&row[1..]) {
                *slot = u32::from_str_radix(cell.trim_start_matches("0x"), 16).expect("data/sobol.tsv: a hex number");
            }
        }
        table
    })
}

/// `sobol_burley`: one dimension of a sample. `rev_bit_index` is the sample
/// index with reversed bits, `scramble_seed` a well randomised seed.
pub fn sobol_burley(mut rev_bit_index: u32, dimension: u32, scramble_seed: u32) -> f32 {
    let mut result = 0u32;
    if dimension == 0 {
        // Dimension 0 is the van der Corput sequence; it is reused for
        // padding the most.
        result = rev_bit_index.reverse_bits();
    } else {
        let direction = &table()[dimension as usize];
        let mut i = 0usize;
        while rev_bit_index != 0 {
            let j = rev_bit_index.leading_zeros() as usize;
            result ^= direction[i + j];
            i += j + 1;
            // Two shifts: one by 32 would not shift on x86.
            rev_bit_index <<= j;
            rev_bit_index <<= 1;
        }
    }
    result = reversed_bit_owen(result, scramble_seed).reverse_bits();
    uint_to_float_excl(result)
}

/// Shuffle the index and mask it (`index &= shuffled_index_mask`).
fn shuffle(index: u32, seed: u32, salt: u32, mask: u32) -> u32 {
    reversed_bit_owen(index.reverse_bits(), seed ^ salt) & mask
}

/// `sobol_burley_sample_1D`.
pub fn sobol_burley_sample_1d(index: u32, dimension: u32, mut seed: u32, mask: u32) -> f32 {
    seed ^= hash_hp_uint(dimension);
    let index = shuffle(index, seed, 0xbff9_5bfe, mask);
    sobol_burley(index, 0, seed ^ 0x635c_77bd)
}

/// `sobol_burley_sample_2D`: `dimension_set` picks two dimensions.
pub fn sobol_burley_sample_2d(index: u32, dimension_set: u32, mut seed: u32, mask: u32) -> [f32; 2] {
    seed ^= hash_hp_uint(dimension_set);
    let index = shuffle(index, seed, 0xf8ad_e99a, mask);
    [sobol_burley(index, 0, seed ^ 0xe0aa_af76), sobol_burley(index, 1, seed ^ 0x9496_4d4e)]
}

/// `sobol_burley_sample_3D`.
pub fn sobol_burley_sample_3d(index: u32, dimension_set: u32, mut seed: u32, mask: u32) -> [f32; 3] {
    seed ^= hash_hp_uint(dimension_set);
    let index = shuffle(index, seed, 0xcaa7_26ac, mask);
    [sobol_burley(index, 0, seed ^ 0x9e78_e391), sobol_burley(index, 1, seed ^ 0x67c3_3241), sobol_burley(index, 2, seed ^ 0x78c3_95c5)]
}

/// `sobol_burley_sample_4D`.
pub fn sobol_burley_sample_4d(index: u32, dimension_set: u32, mut seed: u32, mask: u32) -> [f32; 4] {
    seed ^= hash_hp_uint(dimension_set);
    let index = shuffle(index, seed, 0xc2c1_a055, mask);
    [
        sobol_burley(index, 0, seed ^ 0x3946_8210),
        sobol_burley(index, 1, seed ^ 0xe9d8_a845),
        sobol_burley(index, 2, seed ^ 0x5f32_b482),
        sobol_burley(index, 3, seed ^ 0x1524_cc56),
    ]
}

#[cfg(test)]
mod test;
