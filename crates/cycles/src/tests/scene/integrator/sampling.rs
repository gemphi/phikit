// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::util::{hash_uint, hash_uint2, hash_uint3};

fn sampled(sample_count: i32, pattern: &'static str) -> DeviceScene {
    plain(move |integrator| {
        integrator.node.set("aa_samples", Value::Int(sample_count));
        integrator.node.set_enum("sampling_pattern", pattern);
    })
}

#[test]
fn the_sobol_mask_keeps_the_bits_of_the_next_power_of_two_reversed() {
    assert_eq!(int(&sampled(0, "tabulated_sobol"), "sobol_index_mask"), 0);
    assert_eq!(int(&sampled(64, "tabulated_sobol"), "sobol_index_mask") as u32, 0xFC00_0000);
    assert_eq!(int(&sampled(65, "tabulated_sobol"), "sobol_index_mask") as u32, 0xFE00_0000);
}

#[test]
fn the_tabulated_sobol_sequence_is_between_256_and_8192_samples() {
    let size = [0, 64, 1024, 1025, 100_000].map(|sample_count| int(&sampled(sample_count, "tabulated_sobol"), "tabulated_sobol_sequence_size"));
    assert_eq!(size, [256, 256, 1024, 2048, 8192]);
    // Another pattern has no table.
    assert_eq!(int(&sampled(64, "sobol_burley"), "tabulated_sobol_sequence_size"), 0);
}

#[test]
fn the_rounding_blue_noise_is_the_pure_one_over_a_power_of_two() {
    let odd = sampled(100, "blue_noise_round");
    assert_eq!((int(&odd, "sampling_pattern"), int(&odd, "blue_noise_sequence_length")), (2, 128));
    let even = sampled(64, "blue_noise_round");
    assert_eq!((int(&even, "sampling_pattern"), int(&even, "blue_noise_sequence_length")), (2, 64));
}

#[test]
fn the_first_sample_blue_noise_keeps_one_sample_apart() {
    let first = sampled(64, "blue_noise_first");
    assert_eq!((int(&first, "sampling_pattern"), int(&first, "blue_noise_sequence_length")), (3, 63));
}

#[test]
fn blue_noise_gets_a_hashed_seed_and_sobol_the_seed_itself() {
    let seeded = |pattern: &'static str| {
        plain(move |integrator| {
            integrator.node.set("seed", Value::Int(7));
            integrator.node.set_enum("sampling_pattern", pattern);
        })
    };
    assert_eq!(int(&seeded("tabulated_sobol"), "seed"), 7);
    assert_eq!(int(&seeded("blue_noise_pure"), "seed") as u32, hash_uint(7));
    assert_eq!(int(&seeded("blue_noise_round"), "seed") as u32, hash_uint(7));
}

#[test]
fn without_pixel_jitter_the_kernel_is_told_so() {
    assert_eq!(plain(|_| {}).data.float2("integrator", "pixel_jitter"), [f32::MAX; 2]);
}

#[test]
fn pixel_jitter_steps_through_the_halton_sequence_frame_by_frame() {
    let (mut integrator, mut dscene) = updated(
        |integrator| {
            integrator.node.set("use_pixel_jitter", Value::Boolean(true));
        },
        IntegratorInput::default(),
    );
    assert_eq!(dscene.data.float2("integrator", "pixel_jitter"), [0.0, 1.0 / 3.0 - 0.5]);
    // The seed is hashed with the state the frame started from.
    assert_eq!(int(&dscene, "seed") as u32, hash_uint3(0, 0, 0));
    integrator.node.tag_modified();
    integrator.device_update(&mut dscene, &IntegratorInput::default());
    assert_eq!(dscene.data.float2("integrator", "pixel_jitter"), [-0.25, 2.0 / 3.0 - 0.5]);
    assert_eq!(int(&dscene, "seed") as u32, hash_uint3(0, 1, 1));
}

#[test]
fn a_custom_jitter_is_taken_as_given_and_counts_the_frames() {
    let (integrator, dscene) = updated(
        |integrator| {
            integrator.node.set("use_pixel_jitter", Value::Boolean(true));
            integrator.node.set("use_custom_pixel_jitter_sample", Value::Boolean(true));
            integrator.node.set("custom_pixel_jitter_sample", Value::FloatArray(vec![0.1, 0.2]));
        },
        IntegratorInput::default(),
    );
    assert_eq!(dscene.data.float2("integrator", "pixel_jitter"), [0.1, 0.2]);
    assert_eq!(int(&dscene, "seed") as u32, hash_uint2(0, 0));
    assert_eq!(integrator.pixel_jitter_frame, 1);
}

#[test]
fn switching_jitter_off_starts_the_sequence_over() {
    let (mut integrator, mut dscene) = updated(
        |integrator| {
            integrator.node.set("use_pixel_jitter", Value::Boolean(true));
        },
        IntegratorInput::default(),
    );
    assert_ne!(integrator.pixel_jitter_state, HaltonSequence::default());
    integrator.node.set("use_pixel_jitter", Value::Boolean(false));
    integrator.device_update(&mut dscene, &IntegratorInput::default());
    assert_eq!(integrator.pixel_jitter_state, HaltonSequence::default());
}
