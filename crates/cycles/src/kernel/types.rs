// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h`: the flags of a path and of a
//! shading point, and the ray, the intersection and the shader data a
//! kernel passes about. `data/constant.tsv` holds every enumerator of that
//! header, written by `tool/constant.py`; the constants here are the ones
//! the CPU kernels use, named as there, and `tests/kernel/types.rs` checks
//! each against the table.

macro_rules! constants {
    ($($name:ident = $value:expr),* $(,)?) => {
        $(pub const $name: u32 = $value;)*
        /// Each constant above with its Cycles name, to check against
        /// `data/constant.tsv`.
        pub const TABLE: &[(&str, u32)] = &[$((stringify!($name), $value)),*];
    };
}

constants! {
    PATH_RAY_VISIBILITY_CAMERA = 1 << 0,
    PATH_RAY_VISIBILITY_TRANSMIT = 1 << 1,
    PATH_RAY_VISIBILITY_DIFFUSE = 1 << 2,
    PATH_RAY_VISIBILITY_GLOSSY = 1 << 3,
    PATH_RAY_VISIBILITY_VOLUME_SCATTER = 1 << 4,
    PATH_RAY_VISIBILITY_SHADOW_OPAQUE = 1 << 5,
    PATH_RAY_VISIBILITY_ALL = (1 << 7) - 1,
    PATH_RAY_REFLECT = 1 << 0,
    PATH_RAY_SINGULAR = 1 << 1,
    PATH_RAY_TRANSPARENT = 1 << 2,
    PATH_RAY_IMPORTANCE_BAKE = 1 << 3,
    PATH_RAY_DIFFUSE_ANCESTOR = 1 << 4,
    PATH_RAY_EMISSION = 1 << 5,
    PATH_RAY_MIS_HAD_TRANSMISSION = 1 << 6,
    PATH_RAY_MIS_SKIP = 1 << 7,
    PATH_RAY_TRANSPARENT_BACKGROUND = 1 << 9,
    PATH_RAY_TERMINATE_ON_NEXT_SURFACE = 1 << 10,
    PATH_RAY_TERMINATE_IN_NEXT_VOLUME = 1 << 11,
    PATH_RAY_TERMINATE_AFTER_TRANSPARENT = 1 << 12,
    PATH_RAY_TERMINATE_AFTER_VOLUME = 1 << 13,
    PATH_RAY_TERMINATE = (1 << 10) | (1 << 11) | (1 << 12) | (1 << 13),
    PATH_RAY_SURFACE_PASS = 1 << 18,
    PATH_RAY_VOLUME_PASS = 1 << 19,
    PATH_RAY_ANY_PASS = (1 << 18) | (1 << 19),
    LABEL_TRANSMIT = 1,
    LABEL_REFLECT = 2,
    LABEL_DIFFUSE = 4,
    LABEL_GLOSSY = 8,
    LABEL_SINGULAR = 16,
    LABEL_TRANSPARENT = 32,
    LABEL_VOLUME_SCATTER = 64,
    LABEL_TRANSMIT_TRANSPARENT = 128,
    LABEL_RAY_PORTAL = 512,
    PRNG_FILTER = 0,
    PRNG_LENS_TIME = 1,
    PRNG_TERMINATE = 0,
    PRNG_SURFACE_BSDF = 3,
    PRNG_BOUNCE_NUM = 16,
    PRIMITIVE_TRIANGLE = 1 << 0,
    SD_BACKFACING = 1 << 0,
    SD_EMISSION = 1 << 1,
    SD_BSDF = 1 << 2,
    SD_BSDF_HAS_EVAL = 1 << 3,
    SD_HOLDOUT = 1 << 5,
    SD_TRANSPARENT = 1 << 9,
    SD_BSDF_HAS_TRANSMISSION = 1 << 10,
    SD_HAS_CONSTANT_EMISSION = 1 << 27,
    SD_HAS_EMISSION = 1 << 29,
    SD_MIS_FRONT = 1 << 16,
    SD_MIS_BACK = 1 << 31,
    SD_OBJECT_MOTION = 1 << 1,
    SD_OBJECT_TRANSFORM_APPLIED = 1 << 2,
    SD_OBJECT_NEGATIVE_SCALE = 1 << 3,
    SD_OBJECT_HAS_CORNER_NORMALS = 1 << 12,
    SHADER_SMOOTH_NORMAL = 1 << 31,
    SHADER_CAST_SHADOW = 1 << 30,
    SHADER_USE_MIS = 1 << 28,
    SHADER_EXCLUDE_DIFFUSE = 1 << 27,
    SHADER_EXCLUDE_GLOSSY = 1 << 26,
    SHADER_EXCLUDE_TRANSMIT = 1 << 25,
    SHADER_EXCLUDE_CAMERA = 1 << 24,
    SHADER_EXCLUDE_SCATTER = 1 << 23,
    SHADER_EXCLUDE_ANY = (1 << 27) | (1 << 26) | (1 << 25) | (1 << 24) | (1 << 23) | (1 << 22),
    SHADER_MASK = !((1 << 31) | (1 << 30) | (1 << 28) | (1 << 27) | (1 << 26) | (1 << 25) | (1 << 24) | (1 << 23) | (1 << 22)),
}

/// `OBJECT_NONE`, `PRIM_NONE` and `SHADER_NONE` are `~0`.
pub const NONE: i32 = -1;

/// `SVM_STACK_SIZE` (`kernel/svm/types.h`).
pub const SVM_STACK_SIZE: usize = 255;

/// The most closures a shading point holds on the CPU here. Cycles sizes
/// it by `kernel_data.max_closures`; gate 1's shaders have one.
pub const MAX_CLOSURE: usize = 8;

/// `RaySelfPrimitives`: the primitives a ray must not hit again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaySelf {
    pub object: i32,
    pub prim: i32,
    pub light_object: i32,
    pub light_prim: i32,
}

impl Default for RaySelf {
    fn default() -> Self {
        Self { object: NONE, prim: NONE, light_object: NONE, light_prim: NONE }
    }
}

/// `Ray`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ray {
    pub p: [f32; 3],
    pub d: [f32; 3],
    pub tmin: f32,
    pub tmax: f32,
    pub time: f32,
    pub dp: f32,
    pub dd: f32,
    pub kind: RaySelf,
}

/// `Intersection`: what a ray hit, with `u` and `v` the barycentric
/// coordinates of the hit in the triangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intersection {
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub prim: i32,
    pub object: i32,
    pub kind: i32,
}

impl Default for Intersection {
    fn default() -> Self {
        Self { t: 0.0, u: 0.0, v: 0.0, prim: NONE, object: NONE, kind: 0 }
    }
}

/// `ShaderClosure`: a weight, the chance it is sampled and a closure
/// type; `n` is the normal a BSDF uses (`DiffuseBsdf` adds nothing else).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShaderClosure {
    pub weight: [f32; 3],
    pub sample_weight: f32,
    pub kind: i32,
    pub n: [f32; 3],
}

/// `ShaderData`, the members a diffuse or emissive surface and the
/// background use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaderData {
    pub p: [f32; 3],
    pub n: [f32; 3],
    pub ng: [f32; 3],
    /// `wi`: towards where the ray came from.
    pub wi: [f32; 3],
    pub shader: i32,
    pub flag: u32,
    pub object_flag: u32,
    pub object: i32,
    pub prim: i32,
    pub kind: i32,
    pub u: f32,
    pub v: f32,
    pub ray_length: f32,
    pub time: f32,
    pub num_closure: usize,
    /// `num_closure_left`: room for closures; none when the shader is run only for its emission.
    pub num_closure_left: usize,
    pub closure: [ShaderClosure; MAX_CLOSURE],
    pub closure_emission_background: [f32; 3],
}

impl Default for ShaderData {
    fn default() -> Self {
        Self {
            p: [0.0; 3],
            n: [0.0; 3],
            ng: [0.0; 3],
            wi: [0.0; 3],
            shader: NONE,
            flag: 0,
            object_flag: 0,
            object: NONE,
            prim: NONE,
            kind: 0,
            u: 0.0,
            v: 0.0,
            ray_length: 0.0,
            time: 0.0,
            num_closure: 0,
            num_closure_left: MAX_CLOSURE,
            closure: [ShaderClosure::default(); MAX_CLOSURE],
            closure_emission_background: [0.0; 3],
        }
    }
}
