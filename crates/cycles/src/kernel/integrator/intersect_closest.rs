// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/integrator/intersect_closest.h`
//! (`integrator_intersect_skip_lights` line 20, `integrator_intersect_
//! terminate` 31, `integrator_intersect_next_kernel` 262,
//! `integrator_intersect_closest` 397): trace the path's ray, then queue
//! the kernel that shades what it hit, or the background, or end the path.
//! Lights, volumes, the shadow catcher, MNEE and ambient occlusion
//! bounces are not ported: there is a surface or the background.
use super::path_state::{integrator_path_next, integrator_path_terminate, path_state_continuation_probability, path_state_ray_visibility, path_state_rng_1d, path_state_rng_load};
use super::state::IntegratorState;
use crate::kernel::bvh::scene_intersect;
use crate::kernel::constant;
use crate::kernel::globals::KernelGlobals;
use crate::kernel::types::{Intersection, NONE, PATH_RAY_TERMINATE_ON_NEXT_SURFACE, PRNG_TERMINATE, SD_HAS_EMISSION, SHADER_MASK};
use crate::kernel::DeviceKernel;

/// `integrator_intersect_skip_lights`: when direct light is filtered out
/// for baking, the lights seen on the first bounce are skipped too.
fn skip_lights(kg: &KernelGlobals, state: &IntegratorState) -> bool {
    let direct = constant("FILTER_CLOSURE_DIRECT_LIGHT").unwrap_or(0) as i32;
    kg.integrator.filter_closures & direct != 0 && state.path.bounce == 1
}

/// `integrator_intersect_terminate`: Russian roulette, here in the kernel
/// so a path that ends does not get its shader evaluated. A path that
/// ends on a surface that emits goes on to that shader, which ends it
/// there. True when the path ends.
fn terminate(kg: &KernelGlobals, state: &mut IntegratorState, shader_flags: u32) -> bool {
    let rng = path_state_rng_load(state);
    let path_flag = state.path.flag;
    let continuation_probability = path_state_continuation_probability(kg, state, path_flag);
    state.path.continuation_probability = continuation_probability;
    if continuation_probability != 1.0 {
        let terminate = path_state_rng_1d(kg, &rng, PRNG_TERMINATE);
        if continuation_probability == 0.0 || terminate >= continuation_probability {
            if shader_flags & SD_HAS_EMISSION != 0 {
                // Mark the path to end right after the shader on the surface.
                state.path.flag |= PATH_RAY_TERMINATE_ON_NEXT_SURFACE;
            } else {
                return true;
            }
        }
    }
    false
}

/// `integrator_intersect_next_kernel`.
fn next_kernel(kg: &KernelGlobals, state: &mut IntegratorState, isect: &Intersection, hit: bool) {
    if hit {
        let shader = kg.scene.tri_shader[isect.prim as usize] & SHADER_MASK;
        let flags = kg.shader[shader as usize].flags;
        match terminate(kg, state, flags) {
            false => integrator_path_next(state, DeviceKernel::IntegratorShadeSurface),
            true => integrator_path_terminate(state),
        }
    } else if skip_lights(kg, state) {
        integrator_path_terminate(state);
    } else {
        integrator_path_next(state, DeviceKernel::IntegratorShadeBackground);
    }
}

/// `integrator_intersect_closest`.
pub fn integrator_intersect_closest(kg: &KernelGlobals, state: &mut IntegratorState) {
    let mut ray = state.read_ray();
    let visibility = path_state_ray_visibility(state);
    let (last_prim, last_object) = (state.isect.prim, state.isect.object);
    // The primitive the ray left is not hit again.
    ray.kind.object = last_object;
    ray.kind.prim = last_prim;
    ray.kind.light_object = NONE;
    ray.kind.light_prim = NONE;
    let mut isect = Intersection::default();
    let hit = scene_intersect(kg, &ray, visibility, &mut isect);
    if !hit {
        isect.prim = NONE;
    }
    state.write_isect(&isect);
    next_kernel(kg, state, &isect, hit);
}
