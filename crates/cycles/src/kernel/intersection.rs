// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h` (`Intersection`, lines
//! 614-619): what a ray hit.
use super::types::NONE;
/// `Intersection`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intersection {
    /// How far along the ray.
    pub t: f32,
    /// Where on the primitive: for a triangle, the weights of its second
    /// and third corners; the rest is the first's.
    pub u: f32,
    pub v: f32,
    /// The primitive, in the scene's arrays.
    pub prim: i32,
    pub object: i32,
    /// `type`: a `PrimitiveType`.
    pub kind: i32,
}
impl Default for Intersection {
    /// Nothing hit: `PRIM_NONE` and `OBJECT_NONE`, as a walk starts.
    fn default() -> Self {
        Self { t: 0.0, u: 0.0, v: 0.0, prim: NONE, object: NONE, kind: 0 }
    }
}
