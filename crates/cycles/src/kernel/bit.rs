// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h`: the bits of a path, a
//! label, a shading point and a shader, the numbers a kernel draws by, and
//! the limits. `data/constant.tsv` holds every enumerator of that header,
//! written by `tool/constant.py`; the constants here are the ones the CPU
//! kernels use, named as there, and `kernel/bit/test.rs` checks each
//! against the table. The ray and the hit of the same header are `ray.rs`
//! and `intersection.rs`; the shading point is `shader.rs`.
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

#[cfg(test)]
mod test;
