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
