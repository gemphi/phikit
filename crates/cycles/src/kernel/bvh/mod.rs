// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/bvh/`: walking the tree the `bvh`
//! module built and `DeviceScene` holds.
//!
//! | File | Ported from |
//! |---|---|
//! | `nodes.rs` | `nodes.h`: a ray against the two boxes of an inner node |
//! | `traversal.rs` | `traversal.h`: the walk, into instances and out |
//! | `util.rs` | `util.h`: `intersection_ray_valid`, `ray_offset`, skipping the primitive a ray left |
//! | `scene.rs` | `bvh.h`: `scene_intersect` |
//!
//! The walk is the one of `BVH_FEATURE(0)`: triangles, instances, aligned
//! nodes. Motion blur, hair and point clouds (the `BVH_MOTION`,
//! `BVH_HAIR`, `BVH_POINTCLOUD` variants) are not ported, nor the shadow,
//! local and volume walks.
mod nodes;
mod scene;
mod traversal;
mod util;
pub use scene::scene_intersect;
pub use traversal::bvh_intersect;
pub use util::{intersection_ray_valid, intersection_skip_self_shadow, intersection_t_offset, ray_offset};
