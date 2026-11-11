// SPDX-License-Identifier: Apache-2.0
//! What a device is asked for: `intern/cycles/device`. So far only the
//! denoiser's settings, which the scheduler plans around. The device itself
//! is wgpu, opened by phiray.

mod denoise;

pub use denoise::{DenoiseParam, DenoiserPass, DenoiserPrefilter, DenoiserQuality, DenoiserType};
