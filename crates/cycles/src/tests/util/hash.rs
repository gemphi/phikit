// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

// The expected numbers are from lookup3's final mix written out apart from
// this port, in Python, from `util/hash.h`.

#[test]
fn one_key() {
    assert_eq!([hash_uint(0), hash_uint(1), hash_uint(u32::MAX)], [2_501_500_005, 1_255_655_072, 1_116_397_369]);
}

#[test]
fn two_keys_in_their_order() {
    assert_eq!([hash_uint2(1, 2), hash_uint2(2, 1)], [272_092_061, 2_037_820_192]);
}

#[test]
fn three_keys() {
    assert_eq!([hash_uint3(1, 2, 3), hash_uint3(0, 0, 0)], [3_100_736_010, 2_624_515_579]);
}
