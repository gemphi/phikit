// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/geom/shader_data.h`
//! (`shader_setup_from_ray`, line 36; `shader_setup_from_background`,
//! line 190): the shading point of a surface hit and of a background ray.
//! Ray differentials, the `dPdu`/`dPdv` and motion are not set.
use super::object::{object_normal_transform, object_position_transform};
use super::triangle::{triangle_normal, triangle_smooth_normal};
use crate::kernel::globals::KernelGlobals;
use crate::kernel::types::{Intersection, Ray, SD_BACKFACING, SD_OBJECT_TRANSFORM_APPLIED, SHADER_MASK, SHADER_SMOOTH_NORMAL, ShaderData};
use crate::util::vector::dot;

/// `shader_setup_from_ray`: the shading point of a triangle hit. The hit
/// is a static triangle in the object's own space; its point and normals
/// are brought to world space when the object's transform is not applied.
pub fn shader_setup_from_ray(kg: &KernelGlobals, isect: &Intersection, ray: &Ray) -> ShaderData {
    let mut sd = ShaderData {
        u: isect.u,
        v: isect.v,
        ray_length: isect.t,
        kind: isect.kind,
        object: isect.object,
        object_flag: kg.object_flag(isect.object),
        prim: isect.prim,
        time: ray.time,
        wi: ray.d.map(|part| -part),
        ..ShaderData::default()
    };
    // `triangle_shader_setup`: the shader, the point and the normals.
    let shader_word = kg.scene.tri_shader[isect.prim as usize];
    sd.shader = shader_word as i32;
    let [a, b, c] = super::triangle::triangle_vertices(kg, isect.object, isect.prim);
    // `triangle_point_from_uv`: slightly better precision than `w` weighting.
    let point = [0, 1, 2].map(|axis| a[axis] + isect.u * (b[axis] - a[axis]) + isect.v * (c[axis] - a[axis]));
    sd.p = point;
    let ng = triangle_normal(kg, isect.object, isect.prim, sd.object_flag);
    sd.ng = ng;
    sd.n = ng;
    if shader_word & SHADER_SMOOTH_NORMAL != 0 {
        sd.n = triangle_smooth_normal(kg, ng, isect.object, sd.object_flag, isect.prim, isect.u, isect.v);
    }
    if sd.object_flag & SD_OBJECT_TRANSFORM_APPLIED == 0 {
        sd.p = object_position_transform(kg, isect.object, sd.p);
        sd.n = object_normal_transform(kg, isect.object, sd.n);
        sd.ng = object_normal_transform(kg, isect.object, sd.ng);
    }
    // The shader flags, by the shader's index with its bits masked off.
    sd.flag = kg.shader[(sd.shader as u32 & SHADER_MASK) as usize].flags;
    // The backfacing test flips the normals when the ray hits the back.
    if dot(sd.ng, sd.wi) < 0.0 {
        sd.flag |= SD_BACKFACING;
        sd.ng = sd.ng.map(|part| -part);
        sd.n = sd.n.map(|part| -part);
    }
    sd
}

/// `shader_setup_from_background`: the shading point of a ray that hit
/// nothing. `P` is the ray's direction and the normals point back at it.
pub fn shader_setup_from_background(kg: &KernelGlobals, ray: &Ray) -> ShaderData {
    let back = ray.d.map(|part| -part);
    let shader = kg.background.surface_shader;
    ShaderData {
        p: ray.d,
        n: back,
        ng: back,
        wi: back,
        shader,
        flag: kg.shader[(shader as u32 & SHADER_MASK) as usize].flags,
        time: ray.time,
        ray_length: f32::MAX,
        ..ShaderData::default()
    }
}
