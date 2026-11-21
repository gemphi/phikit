// SPDX-FileCopyrightText: 2018-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/murmurhash.cpp` (`util_murmur_hash3`,
//! `util_hash_to_float`): the hash Cryptomatte names objects by. The hash
//! is Austin Appleby's MurmurHash3, which he placed in the public domain.

/// `mm_hash_fmix32`: the last mix, so every bit of the hash depends on
/// every bit of the key.
fn fmix32(mut hash: u32) -> u32 {
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85eb_ca6b);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0xc2b2_ae35);
    hash ^ (hash >> 16)
}

/// One block of four bytes, or the tail, scrambled.
fn scramble(block: u32) -> u32 {
    block.wrapping_mul(0xcc9e_2d51).rotate_left(15).wrapping_mul(0x1b87_3593)
}

/// `util_murmur_hash3`: the 32-bit MurmurHash3 of `key`.
pub fn murmur_hash3(key: &[u8], seed: u32) -> u32 {
    let mut hash = seed;
    let mut block = key.chunks_exact(4);
    for part in &mut block {
        hash ^= scramble(u32::from_le_bytes([part[0], part[1], part[2], part[3]]));
        hash = hash.rotate_left(13).wrapping_mul(5).wrapping_add(0xe654_6b64);
    }
    let tail = block.remainder();
    let last = tail.iter().enumerate().fold(0u32, |last, (index, byte)| last ^ (u32::from(*byte) << (8 * index)));
    hash ^= match tail.is_empty() {
        true => 0,
        false => scramble(last),
    };
    fmix32(hash ^ key.len() as u32)
}

/// `util_hash_to_float`: a hash as the float the Cryptomatte
/// specification 1.0 stores it as, never infinite, zero or not a number.
pub fn hash_to_float(hash: u32) -> f32 {
    let mantissa = hash & ((1 << 23) - 1);
    let exponent = ((hash >> 23) & 0xff).clamp(1, 254);
    f32::from_bits((hash & (1 << 31)) | (exponent << 23) | mantissa)
}
