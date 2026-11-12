// SPDX-License-Identifier: Apache-2.0
use crate::kernel::sample::*;

// The expected numbers are `sobol_burley.h` written out apart from this
// port, in Python, over the direction numbers of `kernel/tables.h`
// (`_temp/sobol_ref.py` at the time of writing).
fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 2e-7
}

#[test]
fn a_sample_is_what_the_header_computes() {
    let [x, y] = sobol_burley_sample_2d(5, 1, 0x1234, u32::MAX);
    assert!(near(x, 0.875_729_56) && near(y, 0.785_750_8), "{x} {y}");
    let [x, y] = sobol_burley_sample_2d(0, 0, 0, u32::MAX);
    assert!(near(x, 0.530_828_5) && near(y, 0.000_162_015_19), "{x} {y}");
    assert!(near(sobol_burley_sample_1d(3, 2, 99, 0xF000_0000), 0.191_043_5));
    let [x, y, z] = sobol_burley_sample_3d(7, 3, 0x00ab_cdef, u32::MAX);
    assert!(near(x, 0.101_994_6) && near(y, 0.525_872_1) && near(z, 0.374_163_27), "{x} {y} {z}");
}

#[test]
fn a_dimension_is_the_xor_of_the_direction_numbers_of_the_index_bits() {
    assert!(near(sobol_burley(1, 1, 0), 0.811_093_27));
    assert!(near(sobol_burley(3, 2, 5), 0.726_549_6));
    assert!(near(sobol_burley(0x8000_0000, 3, 0x77), 0.436_385_07));
}

#[test]
fn the_first_sixteen_samples_of_a_pixel_fill_a_four_by_four_grid_once() {
    // `sobol_index_mask` for 16 samples keeps 4 bits: the samples are a
    // permutation of the first 16 points of a (0,2)-sequence.
    let mask = 15u32.reverse_bits();
    for seed in [0u32, 1, 0xdead_beef] {
        let mut cell = [0u32; 16];
        for index in 0..16 {
            let [x, y] = sobol_burley_sample_2d(index, 0, seed, mask);
            cell[(x * 4.0) as usize * 4 + (y * 4.0) as usize] += 1;
        }
        assert_eq!(cell, [1; 16], "seed {seed}");
        let mut line = [0u32; 16];
        for index in 0..16 {
            line[(sobol_burley_sample_1d(index, 0, seed, mask) * 16.0) as usize] += 1;
        }
        assert_eq!(line, [1; 16]);
    }
}

#[test]
fn the_samples_are_in_the_unit_interval_and_average_one_half() {
    let (mut sum, count) = (0.0f64, 4096u32);
    for index in 0..count {
        let value = sobol_burley_sample_1d(index, 0, 7, u32::MAX);
        assert!((0.0..1.0).contains(&value));
        sum += f64::from(value);
    }
    assert!((sum / f64::from(count) - 0.5).abs() < 0.01);
}

#[test]
fn different_dimensions_and_seeds_are_different_sequences() {
    let a = sobol_burley_sample_2d(3, 0, 5, u32::MAX);
    assert_ne!(a, sobol_burley_sample_2d(3, 1, 5, u32::MAX));
    assert_ne!(a, sobol_burley_sample_2d(3, 0, 6, u32::MAX));
    assert_ne!(sobol_burley_sample_4d(3, 0, 5, u32::MAX)[..2], a);
}
