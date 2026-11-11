// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

#[test]
fn align_up_is_the_next_multiple_of_a_power_of_two() {
    assert_eq!([0, 1, 128, 129].map(|value| align_up(value, 128)), [0, 128, 128, 256]);
}

#[test]
fn divide_up_rounds_a_remainder_up() {
    assert_eq!([0, 1, 64, 65].map(|value| divide_up(value, 64)), [0, 1, 1, 2]);
}

#[test]
fn the_next_power_of_two_is_above_the_value_also_for_a_power_of_two() {
    assert_eq!([0, 1, 2, 3, 4, 63, 64].map(next_power_of_two), [1, 2, 4, 4, 8, 64, 128]);
    // Cycles shifts by 32 here; a processor gives 1.
    assert_eq!(next_power_of_two(u32::MAX), 1);
}

#[test]
fn bits_are_reversed_end_to_end() {
    assert_eq!([0, 1, 63, u32::MAX].map(reverse_integer_bits), [0, 0x8000_0000, 0xFC00_0000, u32::MAX]);
}

#[test]
fn zero_counts_as_a_power_of_two_and_a_negative_number_does_not() {
    assert_eq!([-4, -1, 0, 1, 6, 8].map(is_power_of_two), [false, false, true, true, false, true]);
}
