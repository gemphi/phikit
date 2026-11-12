// SPDX-License-Identifier: Apache-2.0
use crate::kernel::sample::*;
use crate::util::hash_iqnt2d;

fn data(pattern: SamplingPattern) -> SamplerData {
    SamplerData { pattern, seed: 9, sequence_length: 16, index_mask: 15u32.reverse_bits(), sequence_size: 256, scrambling_distance: 1.0 }
}

#[test]
fn a_pattern_is_found_by_the_number_cycles_gives_it() {
    assert_eq!(SamplingPattern::of(0), SamplingPattern::SobolBurley);
    assert_eq!(SamplingPattern::of(1), SamplingPattern::TabulatedSobol);
    assert_eq!(SamplingPattern::of(2), SamplingPattern::BlueNoisePure);
    assert_eq!(SamplingPattern::of(3), SamplingPattern::BlueNoiseFirst);
}

#[test]
fn the_white_noise_samplers_hash_the_pixel_with_the_seed() {
    let sobol = data(SamplingPattern::SobolBurley);
    assert_eq!(path_rng_pixel_init(&sobol, 3, 5), hash_iqnt2d(3, 5) ^ 9);
    assert_eq!(path_rng_pixel_init(&data(SamplingPattern::TabulatedSobol), 3, 5), hash_iqnt2d(3, 5) ^ 9);
}

#[test]
fn the_blue_noise_sampler_gives_each_pixel_a_section_of_one_sequence() {
    let pure = data(SamplingPattern::BlueNoisePure);
    // Index = sample + pixel * length, with seed 0: pixel 2 sample 3 of
    // sections of 16 is index 35, as pixel 4 sample 3 of sections of 8.
    let a = path_rng_2d(&pure, &[], 2, 3, 0);
    let b = path_rng_2d(&SamplerData { sequence_length: 8, ..pure }, &[], 4, 3, 0);
    assert_eq!(a, b);
    assert_ne!(a, path_rng_2d(&pure, &[], 2, 4, 0));
}

#[test]
fn the_first_sample_of_the_first_pattern_is_apart_from_the_rest() {
    let first = data(SamplingPattern::BlueNoiseFirst);
    // Sample 0 uses the pixel as the index of a one sample sequence; the
    // others count from 0 again.
    assert_eq!(path_rng_1d(&first, &[], 3, 1, 0), path_rng_1d(&data(SamplingPattern::BlueNoisePure), &[], 3, 0, 0));
    assert_ne!(path_rng_1d(&first, &[], 3, 0, 0), path_rng_1d(&first, &[], 3, 1, 0));
}

#[test]
fn a_draw_of_a_path_depends_on_its_dimension() {
    let sobol = data(SamplingPattern::SobolBurley);
    let pixel = path_rng_pixel_init(&sobol, 10, 20);
    assert_ne!(path_rng_2d(&sobol, &[], pixel, 2, 0), path_rng_2d(&sobol, &[], pixel, 2, 1));
    assert_ne!(path_rng_3d(&sobol, &[], pixel, 2, 1)[0], path_rng_1d(&sobol, &[], pixel, 2, 1));
}

#[test]
fn the_classic_pattern_reads_the_table_and_the_others_do_not() {
    let tabulated = data(SamplingPattern::TabulatedSobol);
    // A table in which every float is its place: the sample is where it was read.
    let lut: Vec<f32> = (0..256 * 256 * 4).map(|place| (place / 4) as f32).collect();
    let read = path_rng_1d(&tabulated, &lut, 0xabcdef, 5, 2);
    assert_eq!(read, 46_783.0);
    // The same call with another pattern is Sobol-Burley and ignores the table.
    assert!((0.0..1.0).contains(&path_rng_1d(&data(SamplingPattern::SobolBurley), &lut, 0xabcdef, 5, 2)));
}
