// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/math_base.h` (`align_up`, `divide_up`,
//! `next_power_of_two`, `reverse_integer_bits`).

/// `align_up`: the next multiple of `alignment`, a power of two.
pub fn align_up(value: i32, alignment: i32) -> i32 {
    (value + alignment - 1) & !(alignment - 1)
}

/// `divide_up`: `value / by`, rounded up.
pub fn divide_up(value: i32, by: i32) -> i32 {
    (value + by - 1) / by
}

/// `next_power_of_two`: the power of two above `value`; a power of two
/// gives the next one. Past 2^31 Cycles shifts by 32, which a processor
/// takes as a shift by 0: the answer is 1, and it is 1 here.
pub fn next_power_of_two(value: u32) -> u32 {
    match value {
        0 => 1,
        value => 1u32.wrapping_shl(32 - value.leading_zeros()),
    }
}

/// `reverse_integer_bits`.
pub fn reverse_integer_bits(value: u32) -> u32 {
    value.reverse_bits()
}

/// `is_power_of_two` (`util/types_base.h`). Zero counts as one.
pub fn is_power_of_two(value: i32) -> bool {
    value >= 0 && value & value.wrapping_sub(1) == 0
}
