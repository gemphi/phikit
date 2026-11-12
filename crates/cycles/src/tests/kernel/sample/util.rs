// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::kernel::sample::*;

#[test]
fn the_morton_code_interleaves_x_above_y() {
    assert_eq!((morton2d(1, 0), morton2d(0, 1), morton2d(3, 3), morton2d(2, 0)), (2, 1, 15, 8));
    assert_eq!(expand_bits(0xffff), 0x5555_5555);
    // Only the low 16 bits are used.
    assert_eq!(expand_bits(0x1_0001), 1);
}

// The expected numbers are `util.h` written out apart from this port, in
// Python (`_temp/sobol_ref.py`).
#[test]
fn owen_scrambling_of_a_reversed_bit_integer() {
    assert_eq!(reversed_bit_owen(0x1234_5678, 0xdead_beef), 1_636_902_941);
}

#[test]
fn base_four_scrambling_is_a_permutation_of_the_pixels() {
    // 4096 consecutive Morton codes map to 4096 different numbers.
    let mut seen = std::collections::HashSet::new();
    for code in 0..4096u32 {
        assert!(seen.insert(nested_uniform_scramble_base4(code, 0x1234_5678)));
    }
    // And the same seed gives the same number.
    assert_eq!(nested_uniform_scramble_base4(77, 5), nested_uniform_scramble_base4(77, 5));
    assert_ne!(nested_uniform_scramble_base4(77, 5), nested_uniform_scramble_base4(77, 6));
}
