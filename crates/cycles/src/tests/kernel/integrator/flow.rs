// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/integrator/path.rs`: the start of a path, what a
//! bounce does to its state, Russian roulette and the queue.
use crate::kernel::DeviceKernel;
use crate::kernel::integrator::*;
use crate::kernel::bit::*;
use crate::kernel::KernelWorkTile;

use super::super::rig::Rig;

#[test]
fn a_path_starts_at_its_pixel_with_nothing_queued() {
    let mut state = IntegratorState::new();
    state.path.queued_kernel = 5;
    let tile = KernelWorkTile { offset: -10, stride: 20, ..KernelWorkTile::default() };
    path_state_init(&mut state, &tile, 4, 3);
    assert_eq!(state.path.render_pixel_index, (-10 + 4 + 3 * 20) as u32);
    assert_eq!((state.path.queued_kernel, state.shadow.queued_kernel, state.ao.queued_kernel), (0, 0, 0));
}

#[test]
fn a_new_path_is_a_camera_ray_that_skips_mis_and_sees_a_transparent_background() {
    let mut state = IntegratorState::new();
    path_state_init_integrator(&mut state, 7, 1234, [1.0; 3]);
    let path = &state.path;
    assert_eq!((path.sample, path.bounce, path.rng_pixel, path.rng_offset), (7, 0, 1234, PRNG_BOUNCE_NUM));
    assert_eq!(path.visibility, PATH_RAY_VISIBILITY_CAMERA);
    assert_eq!(path.flag, PATH_RAY_MIS_SKIP | PATH_RAY_TRANSPARENT_BACKGROUND);
    assert_eq!((path.continuation_probability, path.min_ray_pdf, path.throughput), (1.0, f32::MAX, [1.0; 3]));
    assert_eq!((state.isect.prim, state.isect.object), (NONE, NONE));
    integrator_path_init(&mut state, DeviceKernel::IntegratorIntersectClosest);
    assert_eq!(state.path.queued_kernel, DeviceKernel::IntegratorIntersectClosest as u32);
    integrator_path_next(&mut state, DeviceKernel::IntegratorShadeSurface);
    assert_eq!(state.path.queued_kernel, DeviceKernel::IntegratorShadeSurface as u32);
    integrator_path_terminate(&mut state);
    assert_eq!(state.path.queued_kernel, 0);
}

#[test]
fn a_diffuse_bounce_counts_changes_the_visibility_and_moves_the_dimensions_on() {
    let kg = Rig::new(8, 8, [0.1; 3]).kernel();
    let mut state = IntegratorState::new();
    path_state_init_integrator(&mut state, 0, 1, [1.0; 3]);
    path_state_next(&kg, &mut state, LABEL_REFLECT | LABEL_DIFFUSE, 0);
    let path = &state.path;
    assert_eq!((path.bounce, path.diffuse_bounce, path.glossy_bounce), (1, 1, 0));
    assert_eq!(path.rng_offset, 2 * PRNG_BOUNCE_NUM);
    assert_eq!(path.visibility, PATH_RAY_VISIBILITY_DIFFUSE);
    assert!(path.flag & PATH_RAY_REFLECT != 0 && path.flag & PATH_RAY_DIFFUSE_ANCESTOR != 0);
    // The MIS skip and the transparent background are over.
    assert_eq!(path.flag & (PATH_RAY_MIS_SKIP | PATH_RAY_TRANSPARENT_BACKGROUND), 0);
    assert!(path.flag & PATH_RAY_SURFACE_PASS != 0);
}

#[test]
fn a_transparent_bounce_is_not_a_bounce() {
    let kg = Rig::new(8, 8, [0.1; 3]).kernel();
    let mut state = IntegratorState::new();
    path_state_init_integrator(&mut state, 0, 1, [1.0; 3]);
    path_state_next(&kg, &mut state, LABEL_TRANSMIT | LABEL_TRANSPARENT, 0);
    assert_eq!((state.path.bounce, state.path.transparent_bounce), (0, 1));
    assert!(state.path.flag & PATH_RAY_TRANSPARENT != 0);
    assert_eq!(state.path.visibility, PATH_RAY_VISIBILITY_CAMERA);
}

#[test]
fn the_last_bounce_is_marked_to_end_after_it() {
    let mut kg = (*Rig::new(8, 8, [0.1; 3]).kernel()).clone();
    kg.integrator.max_bounce = 2;
    let mut state = IntegratorState::new();
    path_state_init_integrator(&mut state, 0, 1, [1.0; 3]);
    path_state_next(&kg, &mut state, LABEL_REFLECT | LABEL_DIFFUSE, 0);
    assert_eq!(state.path.flag & PATH_RAY_TERMINATE_AFTER_TRANSPARENT, 0);
    path_state_next(&kg, &mut state, LABEL_REFLECT | LABEL_DIFFUSE, 0);
    assert!(state.path.flag & PATH_RAY_TERMINATE_AFTER_TRANSPARENT != 0);
}

#[test]
fn a_ray_that_transmits_does_not_see_diffuse_or_glossy_visibility() {
    let mut state = IntegratorState::new();
    state.path.visibility = PATH_RAY_VISIBILITY_TRANSMIT | PATH_RAY_VISIBILITY_DIFFUSE | PATH_RAY_VISIBILITY_GLOSSY;
    assert_eq!(path_state_ray_visibility(&state), PATH_RAY_VISIBILITY_TRANSMIT);
    state.path.visibility = PATH_RAY_VISIBILITY_DIFFUSE;
    assert_eq!(path_state_ray_visibility(&state), PATH_RAY_VISIBILITY_DIFFUSE);
}

#[test]
fn a_path_always_goes_on_for_the_first_bounces_then_by_the_root_of_its_throughput() {
    let kg = Rig::new(8, 8, [0.1; 3]).kernel();
    let mut state = IntegratorState::new();
    state.path.throughput = [0.04, 0.16, 0.09];
    state.path.bounce = 1;
    // `min_bounce` is 1: a path in its first bounces is not rouletted.
    assert_eq!(path_state_continuation_probability(&kg, &state, 0), 1.0);
    state.path.bounce = 2;
    assert!((path_state_continuation_probability(&kg, &state, 0) - 0.4).abs() < 1e-6);
    state.path.throughput = [4.0, 1.0, 1.0];
    assert_eq!(path_state_continuation_probability(&kg, &state, 0), 1.0);
    // A transparent bounce counts by its own number.
    state.path.transparent_bounce = 0;
    assert_eq!(path_state_continuation_probability(&kg, &state, PATH_RAY_TRANSPARENT), 1.0);
}

#[test]
fn the_numbers_of_a_path_are_the_sampler_s_for_its_pixel_sample_and_dimension() {
    let kg = Rig::new(8, 8, [0.1; 3]).kernel();
    let mut state = IntegratorState::new();
    path_state_init_integrator(&mut state, 3, 99, [1.0; 3]);
    let rng = path_state_rng_load(&state);
    assert_eq!(rng, RngState { rng_pixel: 99, rng_offset: PRNG_BOUNCE_NUM, sample: 3 });
    let a = path_state_rng_2d(&kg, &rng, 1);
    assert_eq!(a, path_state_rng_2d(&kg, &rng, 1));
    assert_ne!(a, path_state_rng_2d(&kg, &rng, 2));
    assert!((0.0..1.0).contains(&path_state_rng_1d(&kg, &rng, PRNG_TERMINATE)));
    assert!(path_state_rng_3d(&kg, &rng, PRNG_SURFACE_BSDF).iter().all(|v| (0.0..1.0).contains(v)));
}
