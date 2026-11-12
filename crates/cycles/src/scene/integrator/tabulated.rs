// SPDX-FileCopyrightText: 2019-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/tabulated_sobol.cpp`
//! (`tabulated_sobol_generate_4D`, lines 13-60), after "Stochastic
//! Generation of (t, s) Sample Sequences" by Helmer, Christensen and
//! Kensler: the points of one pattern of the tabulated Sobol sampler, 4D,
//! made by cutting the domain into smaller and smaller strata and putting
//! each new point in the one stratum of its cell that is not taken. The
//! `xors` that permute the strata are `data/tabulated.tsv`
//! (`tool/tabulated.py`).
use std::sync::OnceLock;

use crate::scene::DeviceScene;
use crate::util::{hash_hp_float, hash_hp_seeded_uint, row_list};

/// `xors[dimension][log_n]`.
fn xors() -> &'static [[u32; 32]; 4] {
    static TABLE: OnceLock<[[u32; 32]; 4]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [[0u32; 32]; 4];
        for row in row_list(include_str!("../../../data/tabulated.tsv")) {
            let dimension: usize = row[0].parse().expect("data/tabulated.tsv: a dimension");
            for (slot, cell) in table[dimension].iter_mut().zip(&row[1..]) {
                *slot = cell.parse().expect("data/tabulated.tsv: a number");
            }
        }
        table
    })
}

/// `tabulated_sobol_generate_4D`: `size` points, from `rng_seed`.
pub fn tabulated_sobol_generate_4d(size: usize, rng_seed: u32) -> Vec<[f32; 4]> {
    let xors = xors();
    let mut points = vec![[0.0f32; 4]; size];
    if size == 0 {
        return points;
    }
    // Randomise the seed, in case it is incrementing.
    let mut rng_i = hash_hp_seeded_uint(rng_seed, 0x4460_5a73);
    let mut next = || {
        rng_i = rng_i.wrapping_add(1);
        hash_hp_float(rng_i.wrapping_sub(1))
    };
    points[0] = [next(), next(), next(), next()];
    // Subdivide into smaller strata, filling in new points as we go.
    let (mut log_n, mut n) = (0usize, 1usize);
    while n < size {
        let strata_count = (n * 2) as f32;
        for i in 0..n {
            if n + i >= size {
                break;
            }
            // The strata already taken in this cell, and a new point in the others.
            let mut point = [0.0f32; 4];
            for axis in 0..4 {
                let occupied = (points[i ^ xors[axis][log_n] as usize][axis] * strata_count) as u32;
                point[axis] = ((occupied ^ 1) as f32 + next()) / strata_count;
            }
            points[n + i] = point;
        }
        log_n += 1;
        n *= 2;
    }
    points
}

/// What `Integrator::device_update` does with `tabulated_sobol_generate_4D`
/// (`integrator.cpp:363-390`): when the pattern is the tabulated Sobol one,
/// the table of `NUM_TAB_SOBOL_PATTERNS` patterns of `sequence_size` points,
/// each pattern made from its own number. It is made here, apart from the
/// integrator's settings, so that updating the settings stays cheap: a
/// sequence of 8192 samples is eight million points.
pub fn fill_sample_pattern(dscene: &mut DeviceScene) {
    let tabulated = crate::kernel::flag("SAMPLING_PATTERN_TABULATED_SOBOL") as i32;
    let size = dscene.data.int("integrator", "tabulated_sobol_sequence_size").max(0) as usize;
    if dscene.data.int("integrator", "sampling_pattern") != tabulated || size == 0 {
        return;
    }
    let patterns = crate::kernel::constant("NUM_TAB_SOBOL_PATTERNS").unwrap_or(0) as u32;
    let mut table = Vec::with_capacity(size * patterns as usize * 4);
    for pattern in 0..patterns {
        table.extend(tabulated_sobol_generate_4d(size, pattern).into_iter().flatten());
    }
    dscene.sample_pattern_lut = table;
}

#[cfg(test)]
mod test;
