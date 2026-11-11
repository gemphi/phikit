// SPDX-License-Identifier: Apache-2.0
use crate::util::vector::{dot, normalize};
use crate::util::*;

#[test]
fn a_normal_comes_back_from_its_32_bits_almost_as_it_went_in() {
    let all = [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 2.0, 3.0], [-3.0, 1.0, -2.0], [0.3, -0.2, -0.9]];
    for normal in all.map(normalize) {
        let back = PackedNormal::encode(normal).decode();
        // Two 16-bit numbers place it to within a few thousandths of a degree.
        assert!(dot(normal, back) > 0.999_999, "{normal:?} -> {back:?}");
    }
}

#[test]
fn straight_up_is_the_middle_of_both_numbers() {
    let up = PackedNormal::encode([0.0, 0.0, 1.0]);
    assert_eq!((up.0 & 0xffff, up.0 >> 16), (0x8000, 0x8000));
    assert!(dot(up.decode(), [0.0, 0.0, 1.0]) > 0.999_999);
}

#[test]
fn a_normal_need_not_be_of_length_one_to_be_packed() {
    assert_eq!(PackedNormal::encode([0.0, 0.0, 5.0]), PackedNormal::encode([0.0, 0.0, 1.0]));
}

#[test]
fn a_normal_that_is_not_a_number_packs_as_zero() {
    assert_eq!(PackedNormal::encode([f32::NAN; 3]), PackedNormal(0));
}
