// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/integrator/init_from_camera.h`
//! (`integrate_camera_sample` line 18, `integrator_init_from_camera`
//! line 55): start a path at a pixel for a sample. The pixel's first
//! sample goes through the middle of the filter; the others draw from the
//! sampler. A path is not restarted after a cache miss (there is no
//! texture cache), nor is the pixel skipped for adaptive sampling.
use crate::kernel::integrator::path::{integrator_path_init, path_state_init, path_state_init_integrator};
use crate::kernel::integrator::state::IntegratorState;
use crate::kernel::camera::{CameraType, camera_sample};
use crate::kernel::film::film_write_sample;
use crate::kernel::sample::{path_rng_2d, path_rng_3d, path_rng_pixel_init};
use crate::kernel::bit::{PRNG_FILTER, PRNG_LENS_TIME};
use crate::kernel::{DeviceKernel, KernelGlobal, KernelWorkTile, Ray};

/// `integrate_camera_sample`: the camera ray of the pixel for the sample,
/// and the throughput it starts with.
fn integrate_camera_sample(kg: &KernelGlobal, sample: i32, x: i32, y: i32, rng_pixel: u32, ray: &mut Ray) -> [f32; 3] {
    let sampler = &kg.integrator.sampler;
    // Filter sampling.
    let rand_filter = match sample == 0 {
        true => [0.5, 0.5],
        false => path_rng_2d(sampler, &kg.scene.sample_pattern_lut, rng_pixel, sample as u32, PRNG_FILTER),
    };
    // Motion blur (time) and depth of field (lens) sampling: x is time and
    // y, z the lens, which gives better convergence when an object is both
    // moving and out of focus.
    let use_motionblur = kg.cam.shuttertime != -1.0;
    let use_dof = kg.cam.aperturesize > 0.0;
    let use_custom_cam = kg.cam.kind == Some(CameraType::Custom);
    let rand_time_lens = match use_motionblur || use_dof || use_custom_cam {
        true => path_rng_3d(sampler, &kg.scene.sample_pattern_lut, rng_pixel, sample as u32, PRNG_LENS_TIME),
        false => [0.0; 3],
    };
    camera_sample(kg, x, y, rand_filter, rand_time_lens[0], [rand_time_lens[1], rand_time_lens[2]], ray)
}

/// `integrator_init_from_camera`: false when the pixel is finished and
/// takes no more samples. `buffer` is the pixel's own floats.
pub fn integrator_init_from_camera(kg: &KernelGlobal, state: &mut IntegratorState, tile: &KernelWorkTile, buffer: &mut [f32], x: i32, y: i32, scheduled_sample: i32) -> bool {
    let _ = buffer;
    // Just enough state to give the path its pixel.
    path_state_init(state, tile, x, y);
    // Count the sample and get the effective sample of the pixel.
    let sample = film_write_sample(kg, scheduled_sample);
    let rng_pixel = path_rng_pixel_init(&kg.integrator.sampler, x as u32, y as u32);
    let mut ray = Ray::default();
    let throughput = integrate_camera_sample(kg, sample, x, y, rng_pixel, &mut ray);
    if throughput == [0.0; 3] {
        return true;
    }
    state.write_ray(&ray);
    path_state_init_integrator(state, sample, rng_pixel, throughput);
    integrator_path_init(state, DeviceKernel::IntegratorIntersectClosest);
    true
}
