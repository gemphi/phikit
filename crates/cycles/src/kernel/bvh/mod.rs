// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2009-2012 Intel Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! A ray against the scene's tree: `intern/cycles/kernel/bvh`, walking the
//! tree the `bvh` module built and `DeviceScene` holds.
//!
//! | File | Ported from |
//! |---|---|
//! | `mod.rs` | `bvh.h`: `scene_intersect`, `scene_intersect_shadow` |
//! | `data.rs` | `types.h` `KernelBVH`, `bvh/types.h`: what a walk reads |
//! | `node.rs` | `nodes.h`: a ray against the two boxes of an inner node |
//! | `traversal.rs` | `traversal.h`: the walk, into instances and out |
//! | `util.rs` | `util.h`: a valid ray, the offsets, skipping what a ray left |
//!
//! The walk is the one of `BVH_FEATURE(0)`: triangles that do not move,
//! instances, aligned nodes. Motion blur, hair and point clouds (the
//! `BVH_MOTION`, `BVH_HAIR`, `BVH_POINTCLOUD` variants) are not ported, nor
//! the shadow-all, local and volume walks.
mod data;
mod node;
mod traversal;
mod util;
pub use data::KernelBvh;
pub use node::bvh_aligned_node_intersect;
pub use traversal::bvh_intersect;
pub use util::{intersection_ray_valid, intersection_skip_self, intersection_skip_self_shadow, intersection_t_offset, ray_offset};
use super::{Intersection, KernelGlobal, Ray};
/// `scene_intersect` (`kernel/bvh/bvh.h`, lines 224-266): the nearest
/// thing a ray hits, by Cycles' own tree. A ray that is no ray hits
/// nothing. Cycles picks a walk by what the scene has; only the plain one
/// is ported, so motion triangles, curves and points are passed through.
pub fn scene_intersect(kg: &KernelGlobal, ray: &Ray, visibility: u32, isect: &mut Intersection) -> bool {
    intersection_ray_valid(ray) && bvh_intersect(kg, ray, isect, visibility)
}
/// `scene_intersect_shadow` (lines 268-275): whether anything is hit at
/// all. With `PATH_RAY_VISIBILITY_SHADOW_OPAQUE` in `visibility` the walk
/// stops at the first hit.
pub fn scene_intersect_shadow(kg: &KernelGlobal, ray: &Ray, visibility: u32) -> bool {
    scene_intersect(kg, ray, visibility, &mut Intersection::default())
}
