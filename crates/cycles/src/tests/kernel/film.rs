// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/film/`: what a path writes to its pixel.
use super::rig::Rig;
use crate::kernel::film::*;
use crate::kernel::integrator::IntegratorState;

fn state(throughput: [f32; 3], bounce: u32) -> IntegratorState {
    let mut state = IntegratorState::new();
    state.path.throughput = throughput;
    state.path.bounce = bounce;
    state
}

#[test]
fn a_pixel_is_found_by_the_path_s_pixel_index_and_the_pass_stride() {
    let mut rig = Rig::new(8, 8, [0.1; 3]);
    let kg = rig.kernel();
    assert_eq!((kg.film.pass_stride, kg.film.pass_combined), (4, 0));
    let mut buffer = vec![0.0f32; 4 * 10];
    let mut path = IntegratorState::new();
    path.path.render_pixel_index = 3;
    film_pass_pixel_render_buffer(&kg, &path, &mut buffer, 0).fill(1.0);
    assert_eq!(buffer.iter().filter(|value| **value == 1.0).count(), 4);
    assert_eq!(&buffer[12..16], &[1.0; 4]);
    // The buffer of a work that starts at pixel 2.
    assert_eq!(film_pass_pixel_render_buffer(&kg, &path, &mut buffer[8..], 2).len(), 4);
}

#[test]
fn contributions_are_added_to_the_pass() {
    let mut buffer = [1.0f32; 4];
    film_write_pass_spectrum(&mut buffer, [1.0, 2.0, 3.0]);
    assert_eq!(buffer, [2.0, 3.0, 4.0, 1.0]);
    film_write_pass_float4(&mut buffer, [1.0, 1.0, 1.0, 0.5]);
    assert_eq!(buffer, [3.0, 4.0, 5.0, 1.5]);
}

#[test]
fn light_is_made_finite_and_clamped_by_the_sum_of_its_parts() {
    let mut rig = Rig::new(8, 8, [0.1; 3]);
    let kg = rig.kernel();
    // No clamp is set: a limit of no limit.
    assert_eq!(film_clamp_light(&kg, [1.0, 2.0, 3.0], 0), [1.0, 2.0, 3.0]);
    assert_eq!(film_clamp_light(&kg, [f32::NAN, f32::INFINITY, 2.0], 0), [0.0, 0.0, 2.0]);
    let mut clamped = (*kg).clone();
    clamped.integrator.sample_clamp_direct = 3.0;
    clamped.integrator.sample_clamp_indirect = 1.5;
    // The sum 6 is scaled to 3 for direct light, to 1.5 for indirect.
    assert_eq!(film_clamp_light(&clamped, [1.0, 2.0, 3.0], 0), [0.5, 1.0, 1.5]);
    assert_eq!(film_clamp_light(&clamped, [1.0, 2.0, 3.0], 1), [0.25, 0.5, 0.75]);
    assert_eq!(film_clamp_light(&clamped, [0.5, 0.5, 0.5], 0), [0.5; 3]);
}

#[test]
fn the_background_adds_throughput_times_colour_to_the_combined_pass() {
    let mut rig = Rig::new(8, 8, [0.1; 3]);
    let kg = rig.kernel();
    let mut pixel = [0.0f32; 4];
    film_write_background(&kg, &state([0.5, 0.25, 1.0], 1), [0.4, 0.8, 0.2], 0.0, false, &mut pixel);
    assert_eq!(pixel, [0.2, 0.2, 0.2, 0.0]);
    // A ray that sees a transparent background adds transparency only.
    let mut transparent = [0.0f32; 4];
    film_write_background(&kg, &state([0.5; 3], 1), [0.4; 3], 0.5, true, &mut transparent);
    assert_eq!(transparent, [0.0, 0.0, 0.0, 0.5]);
}

#[test]
fn an_emission_is_weighted_by_the_throughput_and_the_mis_weight() {
    let mut rig = Rig::new(8, 8, [0.1; 3]);
    let kg = rig.kernel();
    let mut pixel = [0.0f32; 4];
    film_write_surface_emission(&kg, &state([0.5; 3], 1), [2.0, 4.0, 6.0], 0.5, &mut pixel);
    assert_eq!(pixel, [0.5, 1.0, 1.5, 0.0]);
}

#[test]
fn the_sample_to_write_is_the_one_scheduled() {
    let mut rig = Rig::new(8, 8, [0.1; 3]);
    let kg = rig.kernel();
    assert_eq!(film_write_sample(&kg, 7), 7);
}
