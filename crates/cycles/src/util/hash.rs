// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/hash.h` (`hash_uint`, `hash_uint2`,
//! `hash_uint3`): Bob Jenkins' lookup3 hash, its final mix
//! (<http://burtleburtle.net/bob/c/lookup3.c>).

/// `final(a, b, c)` of lookup3; the hash is `c`.
fn finish(a: u32, b: u32, c: u32) -> u32 {
    let (mut a, mut b, mut c) = (a, b, c);
    c ^= b;
    c = c.wrapping_sub(b.rotate_left(14));
    a ^= c;
    a = a.wrapping_sub(c.rotate_left(11));
    b ^= a;
    b = b.wrapping_sub(a.rotate_left(25));
    c ^= b;
    c = c.wrapping_sub(b.rotate_left(16));
    a ^= c;
    a = a.wrapping_sub(c.rotate_left(4));
    b ^= a;
    b = b.wrapping_sub(a.rotate_left(14));
    c ^= b;
    c.wrapping_sub(b.rotate_left(24))
}

/// What `a`, `b` and `c` start as for `count` keys.
fn seed(count: u32) -> u32 {
    0xdead_beef_u32.wrapping_add(count << 2).wrapping_add(13)
}

/// `hash_uint`.
pub fn hash_uint(kx: u32) -> u32 {
    let start = seed(1);
    finish(start.wrapping_add(kx), start, start)
}

/// `hash_uint2`.
pub fn hash_uint2(kx: u32, ky: u32) -> u32 {
    let start = seed(2);
    finish(start.wrapping_add(kx), start.wrapping_add(ky), start)
}

/// `hash_uint3`.
pub fn hash_uint3(kx: u32, ky: u32, kz: u32) -> u32 {
    let start = seed(3);
    finish(start.wrapping_add(kx), start.wrapping_add(ky), start.wrapping_add(kz))
}
/// `uint_to_float_excl`: `[0, uint_max]` to `[0.0, 1.0)`. Cycles divides
/// by 4294967808 and not 2^32, which would round up to 1.0.
pub fn uint_to_float_excl(n: u32) -> f32 {
    n as f32 * (1.0 / 4_294_967_808.0)
}
/// `hash_hp_uint`: the mixing function of Hash Prospector
/// (<https://github.com/skeeto/hash-prospector>), with input zero not
/// mapping to output zero.
pub fn hash_hp_uint(mut i: u32) -> u32 {
    i ^= i >> 16;
    i = i.wrapping_mul(0x21f0_aaad);
    i ^= i >> 15;
    i = i.wrapping_mul(0xd35a_2d97);
    i ^= i >> 15;
    i ^ 0xe6fe_3beb
}
/// `hash_hp_seeded_uint`: the seed is mixed with itself first so that it
/// does not interact poorly with an incrementing `i`.
pub fn hash_hp_seeded_uint(i: u32, mut seed: u32) -> u32 {
    seed ^= seed << 19;
    hash_hp_uint(i ^ seed)
}
/// `hash_iqnt2d`: the 2D hash recommended by "Hash Functions for GPU
/// Rendering" (JCGT 9/3, 2020).
pub fn hash_iqnt2d(x: u32, y: u32) -> u32 {
    let qx = 1_103_515_245u32.wrapping_mul((x >> 1) ^ y);
    let qy = 1_103_515_245u32.wrapping_mul((y >> 1) ^ x);
    1_103_515_245u32.wrapping_mul(qx ^ (qy >> 3))
}
/// `hash_hp_float`: `hash_hp_uint` as a float in `[0, 1)`.
pub fn hash_hp_float(i: u32) -> f32 {
    uint_to_float_excl(hash_hp_uint(i))
}
/// `hash_wang_seeded_uint`: Thomas Wang's integer hash with a seed.
pub fn hash_wang_seeded_uint(i: u32, seed: u32) -> u32 {
    let mut i = (i ^ 61) ^ seed;
    i = i.wrapping_add(i << 3);
    i ^= i >> 4;
    i.wrapping_mul(0x27d4_eb2d)
}
/// `hash_wang_seeded_float`: in `[0, 1)`.
pub fn hash_wang_seeded_float(i: u32, seed: u32) -> f32 {
    uint_to_float_excl(hash_wang_seeded_uint(i, seed))
}
/// `hash_shuffle_uint`: index `i` of a thing `length` long in a shuffled
/// order (<https://andrew-helmer.github.io/permute/>); a seed gives
/// another shuffle, and an index past `length` repeats the same order.
pub fn hash_shuffle_uint(i: u32, length: u32, seed: u32) -> u32 {
    let mut i = i % length;
    let mask = (1u32 << (32 - (length - 1).leading_zeros())).wrapping_sub(1);
    loop {
        i ^= seed;
        i = i.wrapping_mul(0xe170_893d);
        i ^= seed >> 16;
        i ^= (i & mask) >> 4;
        i ^= seed >> 8;
        i = i.wrapping_mul(0x0929_eb3f);
        i ^= seed >> 23;
        i ^= (i & mask) >> 1;
        i = i.wrapping_mul(1 | seed >> 27);
        i = i.wrapping_mul(0x6935_fa69);
        i ^= (i & mask) >> 11;
        i = i.wrapping_mul(0x74dc_b303);
        i ^= (i & mask) >> 2;
        i = i.wrapping_mul(0x9e50_1cc3);
        i ^= (i & mask) >> 2;
        i = i.wrapping_mul(0xc860_a3df);
        i &= mask;
        i ^= i >> 5;
        if i < length {
            return i;
        }
    }
}