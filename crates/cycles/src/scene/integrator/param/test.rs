// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::integrator::test::*;
use crate::device::{DenoiserPass, DenoiserPrefilter, DenoiserQuality, DenoiserType};

fn with(setup: impl FnOnce(&mut Integrator)) -> Integrator {
    let mut integrator = Integrator::new();
    setup(&mut integrator);
    integrator
}

#[test]
fn adaptive_sampling_as_cycles_starts_it() {
    // A threshold of 0.01 gives 64 samples at least, and is scaled by 5.
    let adaptive = Integrator::new().adaptive_sampling();
    assert_eq!((adaptive.enabled, adaptive.adaptive_step, adaptive.min_sample), (true, 16, 64));
    assert!((adaptive.threshold - 0.05).abs() < 1e-7, "{adaptive:?}");
}

#[test]
fn a_threshold_of_zero_is_worked_out_from_the_samples() {
    let automatic = |sample_count: i32| {
        with(|integrator| {
            integrator.node.set("aa_samples", Value::Int(sample_count));
            integrator.node.set("adaptive_threshold", Value::Float(0.0));
        })
        .adaptive_sampling()
    };
    // One over the samples, but 0.001 at least.
    assert!((automatic(100).threshold - 0.05).abs() < 1e-7);
    assert!((automatic(4000).threshold - 0.005).abs() < 1e-7);
    assert_eq!((automatic(100).min_sample, automatic(4000).min_sample), (64, 128));
}

#[test]
fn a_subset_of_the_samples_lowers_the_threshold() {
    let adaptive = with(|integrator| {
        integrator.node.set("aa_samples", Value::Int(100));
        integrator.node.set("use_sample_subset", Value::Boolean(true));
        integrator.node.set("sample_subset_length", Value::Int(25));
    })
    .adaptive_sampling();
    // A quarter of the samples: half the threshold.
    assert!((adaptive.threshold - 0.025).abs() < 1e-7, "{adaptive:?}");
    assert_eq!(adaptive.min_sample, 79);
}

#[test]
fn the_minimum_given_is_kept_but_four_at_least() {
    let min = |given: i32| {
        with(|integrator| {
            integrator.node.set("adaptive_min_samples", Value::Int(given));
        })
        .adaptive_sampling()
        .min_sample
    };
    assert_eq!((min(2), min(40)), (4, 40));
}

#[test]
fn adaptive_sampling_is_off_when_switched_off_or_when_the_denoiser_upscales() {
    let off = with(|integrator| {
        integrator.node.set("use_adaptive_sampling", Value::Boolean(false));
    });
    assert!(!off.adaptive_sampling().enabled);
    let upscaled = with(|integrator| {
        integrator.node.set("use_denoise", Value::Boolean(true));
        integrator.node.set("denoiser_upscale_factor", Value::Float(2.0));
    });
    assert!(!upscaled.adaptive_sampling().enabled);
}

#[test]
fn the_denoiser_s_settings_as_cycles_starts_them() {
    let param = Integrator::new().denoise_param();
    assert!(!param.enabled && param.use_gpu);
    assert_eq!((param.denoiser, param.prefilter, param.quality), (DenoiserType::OpenImageDenoise, DenoiserPrefilter::Accurate, DenoiserQuality::High));
    assert_eq!((param.pass_mask, param.start_sample, param.upscale_factor), (DenoiserPass::ALBEDO | DenoiserPass::NORMAL, 0, 1.0));
}

#[test]
fn the_denoiser_s_settings_follow_the_sockets() {
    let param = with(|integrator| {
        integrator.node.set("use_denoise", Value::Boolean(true));
        integrator.node.set_enum("denoiser_type", "optix");
        integrator.node.set_enum("denoiser_quality", "fast");
        integrator.node.set("denoise_start_sample", Value::Int(4));
    })
    .denoise_param();
    assert!(param.enabled);
    assert_eq!((param.denoiser, param.quality, param.start_sample), (DenoiserType::Optix, DenoiserQuality::Fast, 4));
}

#[test]
fn guiding_is_on_for_what_is_asked_for_with_a_chance() {
    let integrator = with(|integrator| {
        integrator.node.set("use_guiding", Value::Boolean(true));
        integrator.node.set("volume_guiding_probability", Value::Float(0.0));
    });
    let param = integrator.guiding_param(true);
    assert!(param.enabled && param.use_surface_guiding && !param.use_volume_guiding);
    assert_eq!((param.training_sample_count, param.deterministic, param.sampling_type), (128, true, 1));
    assert!(!integrator.guiding_param(false).enabled);
}
