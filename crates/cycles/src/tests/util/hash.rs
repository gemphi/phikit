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

// The expected numbers are Hash Prospector's mix and `hash_iqnt2d` written
// out apart from this port, in Python, from `util/hash.h`.
#[test]
fn the_hash_prospector_mix_and_its_seeded_form() {
    assert_eq!([hash_hp_uint(0), hash_hp_uint(1), hash_hp_uint(u32::MAX)], [3_875_421_163, 3_761_095_064, 1_061_903_720]);
    assert_eq!([hash_hp_seeded_uint(7, 3), hash_hp_seeded_uint(0, 0xdead_beef)], [568_069_491, 3_768_856_412]);
}
#[test]
fn the_2d_hash_of_a_pixel() {
    assert_eq!([hash_iqnt2d(0, 0), hash_iqnt2d(3, 5), hash_iqnt2d(1920, 1080)], [0, 2_135_835_013, 3_455_705_937]);
}
#[test]
fn a_hash_maps_to_the_unit_interval_without_reaching_one() {
    assert_eq!(uint_to_float_excl(0), 0.0);
    assert!(uint_to_float_excl(u32::MAX) < 1.0);
    assert!((uint_to_float_excl(1 << 31) - 0.5).abs() < 1e-6);
}
// The expected numbers are Thomas Wang's hash and the shuffling hash written
// out apart from this port, in Python (`_temp/tab_ref.py`).
#[test]
fn the_wang_hash_and_the_unit_float_of_the_prospector_hash() {
    assert_eq!([hash_wang_seeded_uint(5, 7), hash_wang_seeded_uint(0, 0xdead_beef)], [3_329_800_580, 2_397_977_380]);
    assert!((hash_hp_float(0) - 0.902_316_7).abs() < 1e-6 && (hash_hp_float(1) - 0.875_698_1).abs() < 1e-6);
    assert!((0.0..1.0).contains(&hash_wang_seeded_float(3, 4)));
}
#[test]
fn the_shuffle_is_a_permutation_of_the_indices_and_repeats_past_the_length() {
    let order: Vec<u32> = (0..20).map(|i| hash_shuffle_uint(i, 20, 99)).collect();
    assert_eq!(order, [12, 19, 2, 0, 17, 6, 7, 15, 8, 3, 9, 11, 14, 5, 10, 1, 13, 18, 16, 4]);
    let mut sorted = order.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, (0..20).collect::<Vec<_>>());
    assert_eq!(hash_shuffle_uint(25, 20, 99), order[5]);
    assert_eq!(hash_shuffle_uint(5, 256, 12345), 128);
}