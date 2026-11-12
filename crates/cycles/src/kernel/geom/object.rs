// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/geom/object.h` (lines 20-330 and
//! 440-500): an object's transform and its inverse, and the direction of a
//! ray as the tree walks into an instance. An object that moves
//! (`SD_OBJECT_MOTION`) is not ported.
use crate::kernel::globals::KernelGlobals;
use crate::util::vector::normalize;

/// `bvh_clamp_direction`: no part of the direction is zero, so its
/// reciprocal is finite.
pub fn bvh_clamp_direction(dir: [f32; 3]) -> [f32; 3] {
    let ooeps = 8.271_806e-25_f32;
    dir.map(|part| if part.abs() > ooeps { part } else { ooeps.copysign(part) })
}

/// `bvh_inverse_direction`: `reciprocal`.
pub fn bvh_inverse_direction(dir: [f32; 3]) -> [f32; 3] {
    dir.map(|part| 1.0 / part)
}

/// `object_position_transform`: object space to world.
pub fn object_position_transform(kg: &KernelGlobals, object: i32, p: [f32; 3]) -> [f32; 3] {
    kg.object[object as usize].tfm.point(p)
}

/// `object_inverse_dir_transform`: a world direction in object space.
pub fn object_inverse_dir_transform(kg: &KernelGlobals, object: i32, d: [f32; 3]) -> [f32; 3] {
    kg.object[object as usize].itfm.direction(d)
}

/// `object_normal_transform`: an object-space normal in world space,
/// by the inverse transform's transpose, normalised
/// (`transform_direction_transposed`).
pub fn object_normal_transform(kg: &KernelGlobals, object: i32, n: [f32; 3]) -> [f32; 3] {
    let t = &kg.object[object as usize].itfm;
    let column = |c: usize| t.x[c] * n[0] + t.y[c] * n[1] + t.z[c] * n[2];
    normalize([column(0), column(1), column(2)])
}
