// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/geom/object.h`:
//! `object_fetch_transform` (lines 33-41) and what a walk of the tree does
//! at an instance, `bvh_clamp_direction`, `bvh_inverse_direction`,
//! `bvh_instance_push` and `bvh_instance_pop` (lines 449-510). An object
//! that moves over the shutter (`bvh_instance_motion_push`) is not ported;
//! the rest of the file comes with shading.

use crate::kernel::{KernelGlobal, Ray, RecordLayout};
use crate::util::Transform;

/// Where a `KernelObject` keeps what the kernels read, in its words:
/// looked up in `data/record.tsv` once, not for every ray.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ObjectField {
    /// The words of one object.
    pub word_count: usize,
    pub tfm: usize,
    pub itfm: usize,
    pub position_offset: usize,
}

impl ObjectField {
    pub fn new() -> ObjectField {
        let layout = RecordLayout::of("KernelObject");
        let word = |name: &str| layout.and_then(|layout| layout.field(name)).map_or(0, |field| field.word);
        ObjectField { word_count: layout.map_or(0, |layout| layout.word_count), tfm: word("tfm"), itfm: word("itfm"), position_offset: word("position_offset") }
    }
}

/// `ObjectTransform`: which of an object's two transforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectTransform {
    /// `OBJECT_TRANSFORM`: from the object to the world.
    Object,
    /// `OBJECT_INVERSE_TRANSFORM`: from the world to the object.
    Inverse,
}

/// `object_fetch_transform`.
pub fn object_fetch_transform(kg: &KernelGlobal, object: i32, which: ObjectTransform) -> Transform {
    let start = object as usize * kg.object.word_count
        + match which {
            ObjectTransform::Object => kg.object.tfm,
            ObjectTransform::Inverse => kg.object.itfm,
        };
    let word = &kg.scene.object.word()[start..start + 12];
    let row = |row: usize| std::array::from_fn(|column| f32::from_bits(word[row * 4 + column]));
    Transform { x: row(0), y: row(1), z: row(2) }
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
pub fn bvh_instance_push(kg: &KernelGlobal, object: i32, ray: &Ray) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let tfm = object_fetch_transform(kg, object, ObjectTransform::Inverse);
    let dir = bvh_clamp_direction(tfm.direction(ray.d));
    (tfm.point(ray.p), dir, bvh_inverse_direction(dir))
}

/// `bvh_instance_pop`: the ray as the world sees it again.
pub fn bvh_instance_pop(ray: &Ray) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let dir = bvh_clamp_direction(ray.d);
    (ray.p, dir, bvh_inverse_direction(dir))
}
