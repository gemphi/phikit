// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/closure/emissive.h` (lines 12-65):
//! the emission and background closures add their weight to the one
//! emission a shading point has; an emission is seen from either side.
use crate::kernel::types::{SD_EMISSION, ShaderData};
use crate::util::vector::dot;

/// `emission_setup`.
pub fn emission_setup(sd: &mut ShaderData, weight: [f32; 3]) {
    add_emission(sd, weight);
}

/// `background_setup`.
pub fn background_setup(sd: &mut ShaderData, weight: [f32; 3]) {
    add_emission(sd, weight);
}

fn add_emission(sd: &mut ShaderData, weight: [f32; 3]) {
    if sd.flag & SD_EMISSION != 0 {
        for (sum, channel) in sd.closure_emission_background.iter_mut().zip(weight) {
            *sum += channel;
        }
    } else {
        sd.flag |= SD_EMISSION;
        sd.closure_emission_background = weight;
    }
}

/// `emissive_pdf`: one unless the ray runs along the surface.
pub fn emissive_pdf(ng: [f32; 3], wi: [f32; 3]) -> f32 {
    match dot(ng, wi).abs() > 0.0 {
        true => 1.0,
        false => 0.0,
    }
}

/// `emissive_simple_eval`.
pub fn emissive_simple_eval(ng: [f32; 3], wi: [f32; 3]) -> [f32; 3] {
    [emissive_pdf(ng, wi); 3]
}
