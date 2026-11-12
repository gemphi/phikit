// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/integrator/shade_surface.h`
//! (`integrate_surface_ray_offset` line 28, `integrate_surface_emission`
//! 123, `integrate_surface_bsdf_bssrdf_bounce` 457, `integrate_surface_
//! terminate` 620, `integrate_surface` 720, `integrator_shade_surface`
//! 855) and `surface_shader.h` (`surface_shader_eval` 1249,
//! `surface_shader_bsdf_sample_closure` 973, `surface_shader_emission`
//! 1201). The path leaves a diffuse surface by one cosine-weighted bounce
//! and gathers an emission it sees. There is no next event estimation yet
//! (`integrate_surface_direct_light` waits for lights, gate 3), so the
//! emission of a surface is added with a MIS weight of one, and the bounce
//! alone finds the light; ambient occlusion, holdout, subsurface and
//! volumes are not ported.
use super::path_state::*;
use super::state::IntegratorState;
use crate::kernel::bvh::ray_offset;
use crate::kernel::closure::{bsdf_sample, closure_is_bsdf, emissive_simple_eval};
use crate::kernel::film::film_write_surface_emission;
use crate::kernel::geom::shader_setup_from_ray;
use crate::kernel::globals::KernelGlobals;
use crate::kernel::svm::svm_eval_nodes;
use crate::kernel::types::*;
use crate::kernel::{DeviceKernel, constant, geom};
use crate::util::vector::normalize;
use crate::util::ray_triangle_intersect_self;

/// `integrate_surface_ray_offset`: the start of a ray leaving the surface.
/// The self-intersection test already skips the primitive; the ray is
/// moved off it only if a neighbour might be hit instead.
fn integrate_surface_ray_offset(kg: &KernelGlobals, sd: &ShaderData, ray_p: [f32; 3], ray_d: [f32; 3]) -> [f32; 3] {
    if sd.kind & PRIMITIVE_TRIANGLE as i32 == 0 {
        return ray_p;
    }
    let verts = geom::triangle_vertices(kg, sd.object, sd.prim);
    let (mut local_p, mut local_d) = (ray_p, ray_d);
    if sd.object_flag & SD_OBJECT_TRANSFORM_APPLIED == 0 {
        let itfm = &kg.object[sd.object as usize].itfm;
        local_p = itfm.point(local_p);
        local_d = itfm.direction(local_d);
    }
    match ray_triangle_intersect_self(local_p, local_d, verts) {
        true => ray_p,
        false => ray_offset(ray_p, sd.ng),
    }
}

/// `surface_shader_eval`: run the surface shader, which leaves its
/// closures in `sd`. A path that is ending, or evaluating emission, keeps
/// no closure.
fn surface_shader_eval(kg: &KernelGlobals, sd: &mut ShaderData, path_flag: u32) {
    sd.num_closure = 0;
    sd.num_closure_left = if path_flag & (PATH_RAY_TERMINATE | PATH_RAY_EMISSION) != 0 { 0 } else { MAX_CLOSURE };
    svm_eval_nodes(kg, sd);
}

/// `surface_shader_prepare_closures`: closures the filter takes out
/// (baking), here the emission only.
fn surface_shader_prepare_closures(kg: &KernelGlobals, sd: &mut ShaderData) {
    if kg.integrator.filter_closures & constant("FILTER_CLOSURE_EMISSION").unwrap_or(1) as i32 != 0 {
        sd.closure_emission_background = [0.0; 3];
    }
}

/// `integrate_surface_emission`: what the surface emits towards the ray.
fn integrate_surface_emission(kg: &KernelGlobals, state: &IntegratorState, sd: &ShaderData, buffer: &mut [f32]) {
    let emission = emissive_simple_eval(sd.ng, sd.wi);
    let l = [0, 1, 2].map(|c| emission[c] * sd.closure_emission_background[c]);
    // `light_sample_mis_weight_forward_surface`: one, as no light is sampled.
    film_write_surface_emission(kg, state, l, 1.0, buffer);
}

/// `integrate_surface_terminate`: end the path when Russian roulette said
/// so, else divide the throughput by the probability of surviving.
fn integrate_surface_terminate(state: &mut IntegratorState, path_flag: u32) -> bool {
    let continuation_probability = match path_flag & PATH_RAY_TERMINATE_ON_NEXT_SURFACE != 0 {
        true => 0.0,
        false => state.path.continuation_probability,
    };
    if continuation_probability == 0.0 {
        return true;
    }
    if continuation_probability != 1.0 {
        state.path.throughput = state.path.throughput.map(|part| part / continuation_probability);
    }
    false
}

/// `integrate_surface_bsdf_bssrdf_bounce`: sample the closure and send the
/// path on in the direction it gave. The label of the bounce, or none.
fn integrate_surface_bounce(kg: &KernelGlobals, state: &mut IntegratorState, sd: &ShaderData) -> u32 {
    if sd.flag & SD_BSDF == 0 {
        return 0;
    }
    let rng = path_state_rng_load(state);
    let rand_bsdf = path_state_rng_3d(kg, &rng, PRNG_SURFACE_BSDF);
    // `surface_shader_bsdf_bssrdf_pick`: with one closure, that one.
    let sc = &sd.closure[0];
    if !closure_is_bsdf(sc.kind) {
        return 0;
    }
    // `surface_shader_bsdf_sample_closure`.
    let sample = bsdf_sample(sd, sc, rand_bsdf);
    if sample.pdf == 0.0 {
        return 0;
    }
    let bsdf_eval = [0, 1, 2].map(|c| sample.eval[c] * sc.weight[c]);
    if bsdf_eval == [0.0; 3] {
        return 0;
    }
    let label = sample.label;
    if label & LABEL_TRANSPARENT != 0 {
        // Only the start distance changes for a transparent bounce.
        state.ray.tmin = crate::kernel::bvh::intersection_t_offset(sd.ray_length);
    } else {
        let d = normalize(sample.wo);
        state.ray.p = integrate_surface_ray_offset(kg, sd, sd.p, d);
        state.ray.d = d;
        state.ray.tmin = 0.0;
        state.ray.tmax = f32::MAX;
        state.ray.dp = 0.0;
        state.ray.dd = 0.0;
    }
    // The throughput.
    let weight = bsdf_eval.map(|part| part / sample.pdf);
    state.path.throughput = [0, 1, 2].map(|c| state.path.throughput[c] * weight[c]);
    // The path state, for the next hit's MIS.
    if label & LABEL_TRANSPARENT == 0 {
        state.path.mis_ray_pdf = sample.pdf;
        state.path.mis_origin_n = sd.n;
        state.path.min_ray_pdf = sample.pdf.min(state.path.min_ray_pdf);
    }
    path_state_next(kg, state, label, sd.flag);
    label
}

/// `integrate_surface`: shade the hit and give the label of the bounce
/// that continues the path, or none if it ends.
fn integrate_surface(kg: &KernelGlobals, state: &mut IntegratorState, buffer: &mut [f32]) -> u32 {
    let (isect, ray) = (state.read_isect(), state.read_ray());
    let mut sd = shader_setup_from_ray(kg, &isect, &ray);
    let path_flag = state.path.flag;
    surface_shader_eval(kg, &mut sd, path_flag);
    surface_shader_prepare_closures(kg, &mut sd);
    if sd.flag & SD_EMISSION != 0 {
        integrate_surface_emission(kg, state, &sd, buffer);
    }
    // Most paths have ended in `intersect_closest`; this is for the
    // emission, and for dividing the throughput by the probability.
    if integrate_surface_terminate(state, path_flag) {
        return 0;
    }
    integrate_surface_bounce(kg, state, &sd)
}

/// `integrator_shade_surface`: queue the next ray, or end the path.
pub fn integrator_shade_surface(kg: &KernelGlobals, state: &mut IntegratorState, buffer: &mut [f32]) {
    match integrate_surface(kg, state, buffer) {
        0 => integrator_path_terminate(state),
        _ => integrator_path_next(state, DeviceKernel::IntegratorIntersectClosest),
    }
}
