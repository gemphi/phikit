// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/integrator.cpp`
//! (`Integrator::get_adaptive_sampling`, `get_denoise_params`,
//! `get_guiding_params`, lines 467-563): the integrator's settings as the
//! scheduler, the denoiser and path guiding take them.

use super::{Integrator, MAX_SAMPLE};
use crate::device::{DenoiseParam, DenoiserPrefilter, DenoiserQuality, DenoiserType};
use crate::integrator::AdaptiveSampling;

/// `GuidingParams` (`integrator/guiding.h`): the settings of path guiding.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GuidingParam {
    /// `use`.
    pub enabled: bool,
    pub use_surface_guiding: bool,
    pub use_volume_guiding: bool,
    /// `type`: a `GuidingDistributionType`.
    pub distribution: i32,
    /// `training_samples`.
    pub training_sample_count: i32,
    pub deterministic: bool,
    /// A `GuidingDirectionalSamplingType`.
    pub sampling_type: i32,
    pub roughness_threshold: f32,
}

impl Integrator {
    /// `get_adaptive_sampling`.
    pub fn adaptive_sampling(&self) -> AdaptiveSampling {
        let node = &self.node;
        // With upscaling there is no sample count pass to stop by.
        let upscaled = node.boolean("use_denoise") && node.float("denoiser_upscale_factor") != 1.0;
        if !node.boolean("use_adaptive_sampling") || upscaled {
            return AdaptiveSampling::default();
        }
        let sample_count = node.int("aa_samples");
        let clamped = sample_count.min(MAX_SAMPLE);
        let given = node.float("adaptive_threshold");
        let automatic = match clamped > 0 && given == 0.0 {
            true => (1.0 / sample_count as f32).max(0.001),
            false => given,
        };
        let (offset, length) = (node.int("sample_subset_offset"), node.int("sample_subset_length"));
        let threshold = match node.boolean("use_sample_subset") && clamped > 0 {
            true => automatic * (((offset + length).min(clamped) - offset).max(0) as f32 / clamped as f32).sqrt(),
            false => automatic,
        };
        // 0.1 gives 32, 0.01 gives 64, 0.001 gives 128: a guess that worked
        // well in test scenes.
        let min_sample = match (threshold > 0.0, node.int("adaptive_min_samples")) {
            (true, 0) => (16.0 / threshold.powf(0.3)).ceil() as i32,
            (_, given) => given,
        };
        // The factor of 5 keeps thresholds close to what they once meant.
        AdaptiveSampling { enabled: true, adaptive_step: 16, min_sample: min_sample.max(4), threshold: threshold * 5.0 }
    }

    /// `get_denoise_params`.
    pub fn denoise_param(&self) -> DenoiseParam {
        let node = &self.node;
        DenoiseParam {
            enabled: node.boolean("use_denoise"),
            denoiser: DenoiserType::of(node.int("denoiser_type")),
            start_sample: node.int("denoise_start_sample"),
            pass_mask: node.int("denoiser_passes"),
            use_gpu: node.boolean("denoise_use_gpu"),
            prefilter: DenoiserPrefilter::of(node.int("denoiser_prefilter")),
            quality: DenoiserQuality::of(node.int("denoiser_quality")),
            upscale_factor: node.float("denoiser_upscale_factor"),
            ..DenoiseParam::default()
        }
    }

    /// `get_guiding_params`. `has_guiding` is the device's.
    pub fn guiding_param(&self, has_guiding: bool) -> GuidingParam {
        let node = &self.node;
        let enabled = node.boolean("use_guiding") && has_guiding;
        let use_surface_guiding = enabled && node.boolean("use_surface_guiding") && node.float("surface_guiding_probability") > 0.0;
        let use_volume_guiding = enabled && node.boolean("use_volume_guiding") && node.float("volume_guiding_probability") > 0.0;
        // The roughness set by the user is squared, to act more linearly.
        let roughness = node.float("guiding_roughness_threshold");
        GuidingParam {
            enabled: use_surface_guiding || use_volume_guiding,
            use_surface_guiding,
            use_volume_guiding,
            distribution: node.int("guiding_distribution_type"),
            training_sample_count: node.int("guiding_training_samples"),
            deterministic: node.boolean("deterministic_guiding"),
            sampling_type: node.int("guiding_directional_sampling_type"),
            roughness_threshold: roughness * roughness,
        }
    }
}
