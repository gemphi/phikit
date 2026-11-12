// SPDX-FileCopyrightText: 2018-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

#[test]
fn the_hash_is_murmur3_s_published_values() {
    // The 32-bit MurmurHash3 test values of the reference implementation.
    assert_eq!(murmur_hash3(b"", 0), 0);
    assert_eq!(murmur_hash3(b"", 1), 0x514e_28b7);
    assert_eq!(murmur_hash3(b"", 0xffff_ffff), 0x81f1_6f39);
    assert_eq!(murmur_hash3(b"test", 0), 0xba6b_d213);
    assert_eq!(murmur_hash3(b"test", 0x9747_b28c), 0x704b_81dc);
    assert_eq!(murmur_hash3(b"Hello, world!", 0), 0xc036_3e43);
    assert_eq!(murmur_hash3(b"The quick brown fox jumps over the lazy dog", 0x9747_b28c), 0x2fa8_26cd);
}

#[test]
fn a_tail_of_one_two_or_three_bytes_counts() {
    let hash = [&b"abcd"[..], b"abcde", b"abcdef", b"abcdefg"].map(|key| murmur_hash3(key, 0));
    assert!(hash.iter().enumerate().all(|(index, one)| hash.iter().skip(index + 1).all(|other| one != other)), "{hash:x?}");
}

#[test]
fn a_hash_as_a_float_is_never_zero_infinite_or_not_a_number() {
    // An exponent of all zeros or all ones is moved one inward.
    for hash in [0, u32::MAX, 0x7f80_0000, 0x0000_0001, 0xff80_0001, 0x1234_5678] {
        let float = hash_to_float(hash);
        assert!(float.is_normal(), "{hash:x} -> {float}");
        assert_eq!(float.to_bits() & 0x807f_ffff, hash & 0x807f_ffff);
    }
    assert_eq!(hash_to_float(0x4000_0000), 2.0);
}
