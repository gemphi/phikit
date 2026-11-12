// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/integrator/shade_background.h`
//! (`integrator_eval_background_shader` line 36, `integrate_background`
//! 98, `integrator_shade_background` 245) and `kernel/light/common.h`
//! (`is_light_shader_visible_to_path`, line 74): what a path sees when its
//! ray leaves the scene. Sun lights, the background light's MIS weight
//! (there are no lights yet: the weight is one), the data and denoising
//! passes and the shadow catcher are not ported.
use crate::kernel::integrator::path::integrator_path_terminate;
use crate::kernel::integrator::state::IntegratorState;
use crate::kernel::film::film_write_background;
use crate::kernel::geom::shader_setup_from_background;
use crate::kernel::KernelGlobal;
use crate::kernel::svm::svm_eval_nodes;
use crate::kernel::bit::*;

/// `is_light_shader_visible_to_path`: whether the light's shader (the
/// world's) is seen by a ray of this visibility and these flags.
pub fn is_light_shader_visible_to_path(shader: i32, path_visibility: u32, path_flag: u32) -> bool {
    let shader = shader as u32;
    if shader & SHADER_EXCLUDE_ANY == 0 {
        return true;
    }
    let hidden = (shader & SHADER_EXCLUDE_DIFFUSE != 0 && path_visibility & PATH_RAY_VISIBILITY_DIFFUSE != 0)
        || (shader & SHADER_EXCLUDE_GLOSSY != 0 && path_visibility & PATH_RAY_VISIBILITY_GLOSSY != 0 && path_flag & PATH_RAY_REFLECT != 0)
        || (shader & SHADER_EXCLUDE_TRANSMIT != 0 && path_visibility & PATH_RAY_VISIBILITY_TRANSMIT != 0)
        || (shader & SHADER_EXCLUDE_CAMERA != 0 && path_visibility & PATH_RAY_VISIBILITY_CAMERA != 0)
        || (shader & SHADER_EXCLUDE_SCATTER != 0 && path_visibility & PATH_RAY_VISIBILITY_VOLUME_SCATTER != 0);
    !hidden
}

/// `integrator_eval_background_shader`: the world's colour for the ray.
/// A constant colour is read from the shader's table; else its nodes run.
pub fn integrator_eval_background_shader(kg: &KernelGlobal, state: &IntegratorState) -> [f32; 3] {
    let shader = kg.background.surface_shader;
    if !is_light_shader_visible_to_path(shader, state.path.visibility, state.path.flag) {
        return [0.0; 3];
    }
    let index = (shader as u32 & SHADER_MASK) as usize;
    // `surface_shader_constant_emission`.
    if kg.shader[index].flags & SD_HAS_CONSTANT_EMISSION != 0 {
        return kg.shader[index].constant_emission;
    }
    let ray = state.read_ray();
    let mut sd = shader_setup_from_background(kg, &ray);
    svm_eval_nodes(kg, &mut sd);
    // `surface_shader_background`.
    match sd.flag & SD_EMISSION != 0 {
        true => sd.closure_emission_background,
        false => [0.0; 3],
    }
}

/// `integrate_background`.
fn integrate_background(kg: &KernelGlobal, state: &IntegratorState, buffer: &mut [f32]) {
    let throughput = state.path.throughput;
    let is_transparent_background_ray = kg.background.transparent && state.path.flag & PATH_RAY_TRANSPARENT_BACKGROUND != 0;
    // A transparent background adds transparency and no colour.
    let transparent = if is_transparent_background_ray { (throughput[0] + throughput[1] + throughput[2]) / 3.0 } else { 0.0 };
    let mut l = [0.0; 3];
    if !is_transparent_background_ray {
        l = integrator_eval_background_shader(kg, state);
        // The MIS weight of the background (`light_sample_mis_weight_forward_
        // background`) is one, as no light is sampled.
    }
    film_write_background(kg, state, l, transparent, is_transparent_background_ray, buffer);
}

/// `integrator_shade_background`: the world's colour into the pixel, and
/// the path is over.
pub fn integrator_shade_background(kg: &KernelGlobal, state: &mut IntegratorState, buffer: &mut [f32]) {
    integrate_background(kg, state, buffer);
    integrator_path_terminate(state);
}
