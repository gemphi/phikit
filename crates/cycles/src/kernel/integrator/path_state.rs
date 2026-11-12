// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/integrator/path_state.h`:
//! `path_state_init` (line 25), `path_state_init_integrator` (37),
//! `path_state_next` (95), `path_state_ray_visibility` (255),
//! `path_state_continuation_probability` (270) and the RNG state of a path
//! (`path_state_rng_*`, 330-400). Volumes, ray portals, path guiding and
//! the shadow catcher are not ported.
use super::state::IntegratorState;
use crate::kernel::KernelGlobals;
use crate::kernel::sample::{path_rng_1d, path_rng_2d, path_rng_3d};
use crate::kernel::types::*;
use crate::kernel::DeviceKernel;
use crate::kernel::KernelWorkTile;

/// `path_state_init`: just enough state for the camera kernel's early
/// outputs: the pixel the path will write to, and no kernel queued.
pub fn path_state_init(state: &mut IntegratorState, tile: &KernelWorkTile, x: i32, y: i32) {
    state.path.render_pixel_index = (tile.offset + x + y * tile.stride as i32) as u32;
    state.path.queued_kernel = 0;
    state.shadow.queued_kernel = 0;
    state.ao.queued_kernel = 0;
}

/// `path_state_init_integrator`: the rest of the state of a new path.
pub fn path_state_init_integrator(state: &mut IntegratorState, sample: i32, rng_pixel: u32, throughput: [f32; 3]) {
    let path = &mut state.path;
    path.sample = sample as u32;
    path.bounce = 0;
    path.diffuse_bounce = 0;
    path.glossy_bounce = 0;
    path.transmission_bounce = 0;
    path.transparent_bounce = 0;
    path.volume_bounce = 0;
    path.volume_bounds_bounce = 0;
    path.rng_pixel = rng_pixel;
    path.rng_offset = PRNG_BOUNCE_NUM;
    path.visibility = PATH_RAY_VISIBILITY_CAMERA;
    path.flag = PATH_RAY_MIS_SKIP | PATH_RAY_TRANSPARENT_BACKGROUND;
    path.mis_ray_pdf = 0.0;
    path.min_ray_pdf = f32::MAX;
    path.continuation_probability = 1.0;
    path.throughput = throughput;
    path.optical_depth = 0.0;
    path.mnee = 0;
    state.clear_isect();
}

/// `path_state_next`: the counters, the flags and the visibility after a
/// bounce with this `label` (`LABEL_*`) off a shader with `shader_flag`.
pub fn path_state_next(kg: &KernelGlobals, state: &mut IntegratorState, label: u32, shader_flag: u32) {
    let integrator = &kg.integrator;
    let mut flag = state.path.flag;
    // A ray through a transparent surface keeps the flags of the one
    // before and is not a regular bounce.
    if label & (LABEL_TRANSPARENT | LABEL_RAY_PORTAL) != 0 {
        let transparent_bounce = state.path.transparent_bounce + 1;
        flag |= PATH_RAY_TRANSPARENT;
        if transparent_bounce as i32 >= integrator.transparent_max_bounce {
            flag |= PATH_RAY_TERMINATE_ON_NEXT_SURFACE;
        }
        state.path.flag = flag;
        state.path.transparent_bounce = transparent_bounce;
        state.path.rng_offset += PRNG_BOUNCE_NUM;
        return;
    }
    let bounce = state.path.bounce + 1;
    if bounce as i32 >= integrator.max_bounce {
        flag |= PATH_RAY_TERMINATE_AFTER_TRANSPARENT;
    }
    let mut visibility = 0;
    flag &= !(PATH_RAY_REFLECT | PATH_RAY_SINGULAR | PATH_RAY_TRANSPARENT | PATH_RAY_IMPORTANCE_BAKE | PATH_RAY_MIS_SKIP | PATH_RAY_MIS_HAD_TRANSMISSION);
    // Surface reflection or transmission.
    if label & LABEL_REFLECT != 0 {
        flag |= PATH_RAY_REFLECT;
        flag &= !PATH_RAY_TRANSPARENT_BACKGROUND;
        if label & LABEL_DIFFUSE != 0 {
            state.path.diffuse_bounce += 1;
            if state.path.diffuse_bounce as i32 >= integrator.max_diffuse_bounce {
                flag |= PATH_RAY_TERMINATE_AFTER_TRANSPARENT;
            }
        } else {
            state.path.glossy_bounce += 1;
            if state.path.glossy_bounce as i32 >= integrator.max_glossy_bounce {
                flag |= PATH_RAY_TERMINATE_AFTER_TRANSPARENT;
            }
        }
    } else {
        visibility |= PATH_RAY_VISIBILITY_TRANSMIT;
        if label & LABEL_TRANSMIT_TRANSPARENT == 0 {
            flag &= !PATH_RAY_TRANSPARENT_BACKGROUND;
        }
        state.path.transmission_bounce += 1;
        if state.path.transmission_bounce as i32 >= integrator.max_transmission_bounce {
            flag |= PATH_RAY_TERMINATE_AFTER_TRANSPARENT;
        }
    }
    // Diffuse, glossy or singular.
    if label & LABEL_DIFFUSE != 0 {
        visibility |= PATH_RAY_VISIBILITY_DIFFUSE;
        flag |= PATH_RAY_DIFFUSE_ANCESTOR;
    } else if label & LABEL_GLOSSY != 0 {
        visibility |= PATH_RAY_VISIBILITY_GLOSSY;
    } else {
        visibility |= PATH_RAY_VISIBILITY_GLOSSY;
        flag |= PATH_RAY_SINGULAR | PATH_RAY_MIS_SKIP;
    }
    // For consistent MIS weights with the light tree.
    if shader_flag & SD_BSDF_HAS_TRANSMISSION != 0 {
        flag |= PATH_RAY_MIS_HAD_TRANSMISSION;
    }
    // The render pass categories.
    if flag & PATH_RAY_ANY_PASS == 0 && flag & PATH_RAY_TRANSPARENT_BACKGROUND == 0 {
        flag |= PATH_RAY_SURFACE_PASS;
    }
    state.path.visibility = visibility;
    state.path.flag = flag;
    state.path.bounce = bounce;
    state.path.rng_offset += PRNG_BOUNCE_NUM;
}

/// `path_state_ray_visibility`: for visibility, diffuse and glossy are for
/// reflection only.
pub fn path_state_ray_visibility(state: &IntegratorState) -> u32 {
    let mut visibility = state.path.visibility;
    if visibility & PATH_RAY_VISIBILITY_TRANSMIT != 0 {
        visibility &= !(PATH_RAY_VISIBILITY_DIFFUSE | PATH_RAY_VISIBILITY_GLOSSY);
    }
    visibility
}

/// `path_state_continuation_probability`: one for the first bounces, else
/// the square root of the throughput's largest part (about the view
/// transform's curve, so paths end a little later).
pub fn path_state_continuation_probability(kg: &KernelGlobals, state: &IntegratorState, path_flag: u32) -> f32 {
    let integrator = &kg.integrator;
    if path_flag & PATH_RAY_TRANSPARENT != 0 {
        if state.path.transparent_bounce as i32 <= integrator.transparent_min_bounce {
            return 1.0;
        }
    } else if state.path.bounce as i32 <= integrator.min_bounce {
        return 1.0;
    }
    let throughput = state.path.throughput;
    let largest = throughput.iter().fold(f32::MIN, |most, part| most.max(part.abs()));
    largest.sqrt().min(1.0)
}

/// `RNGState`: the numbers a path draws are found by the pixel, the
/// dimension offset of the bounce and the sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RngState {
    pub rng_pixel: u32,
    pub rng_offset: u32,
    pub sample: u32,
}

/// `path_state_rng_load`.
pub fn path_state_rng_load(state: &IntegratorState) -> RngState {
    RngState { rng_pixel: state.path.rng_pixel, rng_offset: state.path.rng_offset, sample: state.path.sample }
}

/// `path_state_rng_1D`.
pub fn path_state_rng_1d(kg: &KernelGlobals, rng: &RngState, dimension: u32) -> f32 {
    path_rng_1d(&kg.integrator.sampler, &kg.scene.sample_pattern_lut, rng.rng_pixel, rng.sample, rng.rng_offset + dimension)
}

/// `path_state_rng_2D`.
pub fn path_state_rng_2d(kg: &KernelGlobals, rng: &RngState, dimension: u32) -> [f32; 2] {
    path_rng_2d(&kg.integrator.sampler, &kg.scene.sample_pattern_lut, rng.rng_pixel, rng.sample, rng.rng_offset + dimension)
}

/// `path_state_rng_3D`.
pub fn path_state_rng_3d(kg: &KernelGlobals, rng: &RngState, dimension: u32) -> [f32; 3] {
    path_rng_3d(&kg.integrator.sampler, &kg.scene.sample_pattern_lut, rng.rng_pixel, rng.sample, rng.rng_offset + dimension)
}

/// `integrator_path_init`: queue `next` for the path.
pub fn integrator_path_init(state: &mut IntegratorState, next: DeviceKernel) {
    state.path.queued_kernel = next as u32;
}

/// `integrator_path_next`.
pub fn integrator_path_next(state: &mut IntegratorState, next: DeviceKernel) {
    state.path.queued_kernel = next as u32;
}

/// `integrator_path_terminate`: nothing is queued; the path is done.
pub fn integrator_path_terminate(state: &mut IntegratorState) {
    state.path.queued_kernel = 0;
}
