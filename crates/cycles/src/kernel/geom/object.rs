// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/geom/object.h`:
//! `object_fetch_transform` (lines 33-41), `object_position_transform`,
//! `object_inverse_dir_transform` and `object_normal_transform` (lines
//! 200-330), and what a walk of the tree does at an instance,
//! `bvh_clamp_direction`, `bvh_inverse_direction`, `bvh_instance_push` and
//! `bvh_instance_pop` (lines 449-510). An object that moves over the
//! shutter (`SD_OBJECT_MOTION`, `bvh_instance_motion_push`) is not ported.
use crate::kernel::{KernelGlobals, Ray};
use crate::util::Transform;
use crate::util::vector::normalize;
/// `ObjectTransform`: which of an object's two transforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectTransform {
    /// `OBJECT_TRANSFORM`: from the object to the world.
    Object,
    /// `OBJECT_INVERSE_TRANSFORM`: from the world to the object.
    Inverse,
}
/// `object_fetch_transform`.
pub fn object_fetch_transform(kg: &KernelGlobals, object: i32, which: ObjectTransform) -> Transform {
    let object = &kg.object[object as usize];
    match which {
        ObjectTransform::Object => object.tfm,
        ObjectTransform::Inverse => object.itfm,
    }
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
/// `bvh_clamp_direction`: a direction with no part so near zero that its
/// inverse is infinite.
pub fn bvh_clamp_direction(dir: [f32; 3]) -> [f32; 3] {
    // Cycles' `ooeps`: two to the power of minus eighty.
    let least = 2.0_f32.powi(-80);
    dir.map(|part| match part.abs() > least {
        true => part,
        false => least.copysign(part),
    })
}
/// `bvh_inverse_direction`.
pub fn bvh_inverse_direction(dir: [f32; 3]) -> [f32; 3] {
    dir.map(|part| 1.0 / part)
}
/// `bvh_instance_push`: the ray as the object sees it, to walk the tree of
/// its geometry: where it starts, where it goes, and the inverse of that.
/// The direction is not made of length one again, so a distance along it
/// is still a distance along the ray in the world.
pub fn bvh_instance_push(kg: &KernelGlobals, object: i32, ray: &Ray) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let tfm = object_fetch_transform(kg, object, ObjectTransform::Inverse);
    let dir = bvh_clamp_direction(tfm.direction(ray.d));
    (tfm.point(ray.p), dir, bvh_inverse_direction(dir))
}
/// `bvh_instance_pop`: the ray as the world sees it again.
pub fn bvh_instance_pop(ray: &Ray) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let dir = bvh_clamp_direction(ray.d);
    (ray.p, dir, bvh_inverse_direction(dir))
}
