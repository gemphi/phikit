// SPDX-License-Identifier: Apache-2.0
use crate::kernel::sample::*;

// The expected places are `tabulated_sobol.h` written out apart from this
// port, in Python (`_temp/tab_ref.py` at the time of writing).
fn data(scrambling_distance: f32) -> SamplerData {
    SamplerData { pattern: SamplingPattern::TabulatedSobol, seed: 11, sequence_length: 0, index_mask: 0, sequence_size: 256, scrambling_distance }
}

/// A table in which every float is the number of its point.
fn ramp() -> Vec<f32> {
    (0..256 * 256 * 4).map(|place| (place / 4) as f32).collect()
}

#[test]
fn a_sample_is_the_point_of_the_shuffled_pattern_and_index() {
    let lut = ramp();
    assert_eq!(tabulated_sobol_sample_1d(&data(1.0), &lut, 5, 0x00ab_cdef, 2), 46_783.0);
    assert_eq!(tabulated_sobol_sample_1d(&data(1.0), &lut, 300, 7, 1), 55_989.0);
}

#[test]
fn a_two_or_three_dimensional_sample_reads_the_next_floats_of_the_point() {
    // Each float of a point is its own number plus the axis over ten.
    let lut: Vec<f32> = (0..256 * 256 * 4).map(|place| (place / 4) as f32 + (place % 4) as f32 / 10.0).collect();
    let [x, y] = tabulated_sobol_sample_2d(&data(1.0), &lut, 5, 0x00ab_cdef, 2);
    assert_eq!((x, y), (46_783.0, 46_783.1));
    let [x, y, z] = tabulated_sobol_sample_3d(&data(1.0), &lut, 5, 0x00ab_cdef, 2);
    assert_eq!((x, y, z), (46_783.0, 46_783.1, 46_783.2));
}

#[test]
fn the_pattern_is_the_dimensions_and_the_samples_within_a_sequence_are_a_permutation() {
    let lut = ramp();
    // The first 256 samples of a dimension are the 256 points of one pattern.
    let mut seen = std::collections::HashSet::new();
    let first = tabulated_sobol_sample_1d(&data(1.0), &lut, 0, 99, 3) as u32 / 256;
    for sample in 0..256 {
        let point = tabulated_sobol_sample_1d(&data(1.0), &lut, sample, 99, 3) as u32;
        assert_eq!(point / 256, first);
        assert!(seen.insert(point));
    }
    // Another dimension is another pattern.
    assert_ne!(first, tabulated_sobol_sample_1d(&data(1.0), &lut, 0, 99, 4) as u32 / 256);
}

#[test]
fn a_scrambling_distance_shares_the_sequence_and_rotates_it_a_little_by_the_pixel() {
    let lut: Vec<f32> = (0..256 * 256 * 4).map(|place| ((place / 4) % 100) as f32 / 100.0).collect();
    let near = data(0.1);
    let a = tabulated_sobol_sample_1d(&near, &lut, 5, 111, 2);
    let b = tabulated_sobol_sample_1d(&near, &lut, 5, 222, 2);
    // Two pixels read the same point (the seed is the integrator's) and
    // differ by their rotation, under the distance.
    let d = (a - b).abs();
    assert!(a != b && d.min(1.0 - d) < 0.1 + 1e-6, "{a} {b}");
    assert!((0.0..1.0).contains(&a) && (0.0..1.0).contains(&b));
}

#[test]
fn a_table_that_is_not_there_gives_zero() {
    assert_eq!(tabulated_sobol_sample_1d(&data(1.0), &[], 5, 3, 2), 0.0);
}
