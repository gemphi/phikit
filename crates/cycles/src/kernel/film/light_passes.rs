// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/film/light_passes.h`:
//! `film_clamp_light` (line 128), `film_write_sample` (172),
//! `film_write_combined_transparent_pass` (368), `film_write_transparent`
//! (586), `film_write_background` (620) and `film_write_surface_emission`
//! (680). The shadow catcher, the adaptive buffer, volume scattering
//! guiding and the light, data and lightgroup passes are not written.
//! The kernels write to `buffer`, the floats of the path's own pixel (the
//! `film_pass_pixel_render_buffer` of `write.rs` finds it).

use super::write::{film_write_pass_float4, film_write_pass_spectrum};
use crate::kernel::globals::KernelGlobals;
use crate::kernel::integrator::IntegratorState;

/// `ensure_finite`: a part that is not finite is zero.
fn ensure_finite(l: [f32; 3]) -> [f32; 3] {
    l.map(|part| if part.is_finite() { part } else { 0.0 })
}

/// `film_clamp_light`: the contribution with its parts finite, and scaled
/// down when its sum is over the clamp of direct light (the first bounce)
/// or of indirect light (`bounce`, counted from the camera, zero).
pub fn film_clamp_light(kg: &KernelGlobals, l: [f32; 3], bounce: i32) -> [f32; 3] {
    let l = ensure_finite(l);
    let limit = match bounce > 0 {
        true => kg.integrator.sample_clamp_indirect,
        false => kg.integrator.sample_clamp_direct,
    };
    let sum = l[0].abs() + l[1].abs() + l[2].abs();
    match sum > limit {
        true => l.map(|part| part * (limit / sum)),
        false => l,
    }
}

/// `film_write_sample`: counts the sample and gives the sample number to
/// use. Without a sample count pass it is the scheduled sample.
pub fn film_write_sample(kg: &KernelGlobals, scheduled_sample: i32) -> i32 {
    let _ = kg.film.pass_sample_count;
    scheduled_sample
}

/// `film_write_combined_transparent_pass`: the contribution and the
/// transparency into the combined pass.
pub fn film_write_combined_transparent_pass(kg: &KernelGlobals, buffer: &mut [f32], contribution: [f32; 3], transparent: f32) {
    let combined = kg.film.pass_combined as usize;
    film_write_pass_float4(&mut buffer[combined..], [contribution[0], contribution[1], contribution[2], transparent]);
}

/// `film_write_combined_pass`: the contribution to the combined pass.
fn film_write_combined_pass(kg: &KernelGlobals, buffer: &mut [f32], contribution: [f32; 3]) {
    film_write_pass_spectrum(&mut buffer[kg.film.pass_combined as usize..], contribution);
}

/// `film_write_transparent`: transparency is accumulated, not alpha, so a
/// path need not write alpha when it ends.
fn film_write_transparent(kg: &KernelGlobals, buffer: &mut [f32], transparent: f32) {
    buffer[kg.film.pass_combined as usize + 3] += transparent;
}

/// `film_write_background`: what a path that left the scene sees. For a
/// transparent background ray only the transparency is written.
pub fn film_write_background(kg: &KernelGlobals, state: &IntegratorState, l: [f32; 3], transparent: f32, is_transparent_background_ray: bool, buffer: &mut [f32]) {
    let throughput = state.path.throughput;
    let contribution = film_clamp_light(kg, [throughput[0] * l[0], throughput[1] * l[1], throughput[2] * l[2]], state.path.bounce as i32 - 1);
    match is_transparent_background_ray {
        true => film_write_transparent(kg, buffer, transparent),
        false => film_write_combined_transparent_pass(kg, buffer, contribution, transparent),
    }
}

/// `film_write_surface_emission`: the emission of a surface, weighted by
/// the path's throughput and the MIS weight.
pub fn film_write_surface_emission(kg: &KernelGlobals, state: &IntegratorState, l: [f32; 3], mis_weight: f32, buffer: &mut [f32]) {
    let throughput = state.path.throughput;
    let contribution = [0, 1, 2].map(|c| throughput[c] * l[c] * mis_weight);
    let contribution = film_clamp_light(kg, contribution, state.path.bounce as i32 - 1);
    film_write_combined_pass(kg, buffer, contribution);
}
