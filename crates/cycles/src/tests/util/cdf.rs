// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

#[test]
fn the_sum_of_a_constant_rises_evenly_to_one() {
    assert_eq!(cdf_evaluate(4, 0.0, 1.0, |_| 3.0), [0.0, 0.25, 0.5, 0.75, 1.0]);
}

#[test]
fn the_sum_of_nothing_is_zero_and_still_ends_at_one() {
    assert_eq!(cdf_evaluate(3, 0.0, 1.0, |_| 0.0), [0.0, 0.0, 0.0, 1.0]);
}

#[test]
fn a_negative_function_counts_by_its_size() {
    assert_eq!(cdf_evaluate(2, 0.0, 1.0, |x| x - 0.5), [0.0, 0.5, 1.0]);
}

#[test]
fn inverting_an_even_rise_gives_the_numbers_back() {
    let cdf = [0.0, 0.25, 0.5, 0.75, 1.0];
    assert_eq!(cdf_invert(4, 0.0, 1.0, &cdf, false), [0.125, 0.375, 0.625, 0.875]);
    assert_eq!(cdf_invert(4, 2.0, 4.0, &cdf, false), [2.25, 2.75, 3.25, 3.75]);
}

#[test]
fn a_steep_part_of_the_sum_draws_more_of_the_numbers() {
    // All of the function is in the first half of the range.
    let drawn = cdf_inverted(64, 0.0, 1.0, |x| f32::from(x < 0.5), false);
    assert!(drawn.iter().all(|place| *place <= 0.51), "{drawn:?}");
}

#[test]
fn the_symmetric_table_mirrors_about_a_half() {
    let table = cdf_inverted(10, 0.0, 0.5, |_| 1.0, true);
    for index in 0..=4 {
        assert!((table[4 + index] + table[4 - index] - 1.0).abs() < 1e-6, "{table:?}");
    }
    // An even table has one entry the mirroring never reaches.
    assert_eq!((table[4], table[9]), (0.5, 0.0));
}
