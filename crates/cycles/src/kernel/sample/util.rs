// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/sample/util.h` (lines 18-80): Owen
//! scrambling of a reversed-bit integer and the Morton code of a pixel.

/// `reversed_bit_owen`: base-2 Owen scrambling of a reversed-bit integer,
/// equivalent to the Laine-Karras permutation but of higher quality
/// (<https://psychopath.io/post/2021_01_30_building_a_better_lk_hash>).
pub fn reversed_bit_owen(mut n: u32, seed: u32) -> u32 {
    n ^= n.wrapping_mul(0x3d20_adea);
    n = n.wrapping_add(seed);
    n = n.wrapping_mul((seed >> 16) | 1);
    n ^= n.wrapping_mul(0x0552_6c56);
    n ^= n.wrapping_mul(0x53a2_2864);
    n
}
/// `reversed_bit_owen_base4`
/// (<https://psychopath.io/post/2022_08_14_a_fast_hash_for_base_4_owen_scrambling>).
pub fn reversed_bit_owen_base4(mut n: u32, seed: u32) -> u32 {
    n ^= n.wrapping_mul(0x3d20_adea);
    n ^= (n >> 1) & (n << 1) & 0x5555_5555;
    n = n.wrapping_add(seed);
    n = n.wrapping_mul((seed >> 16) | 1);
    n ^= (n >> 1) & (n << 1) & 0x5555_5555;
    n ^= n.wrapping_mul(0x0552_6c56);
    n ^= n.wrapping_mul(0x53a2_2864);
    n
}
/// `nested_uniform_scramble_base4`: base-4 Owen scrambling of an integer.
pub fn nested_uniform_scramble_base4(i: u32, seed: u32) -> u32 {
    reversed_bit_owen_base4(i.reverse_bits(), seed).reverse_bits()
}
/// `expand_bits`: the low 16 bits of `x`, a zero between each.
pub fn expand_bits(mut x: u32) -> u32 {
    x &= 0x0000_ffff;
    x = (x ^ (x << 8)) & 0x00ff_00ff;
    x = (x ^ (x << 4)) & 0x0f0f_0f0f;
    x = (x ^ (x << 2)) & 0x3333_3333;
    x = (x ^ (x << 1)) & 0x5555_5555;
    x
}
/// `morton2d`.
pub fn morton2d(x: u32, y: u32) -> u32 {
    (expand_bits(x) << 1) | expand_bits(y)
}
