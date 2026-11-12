// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! The shapes a ray meets: `intern/cycles/kernel/geom`, the geometry of a
//! hit read out of the device scene.
//!
//! | File | Ported from |
//! |---|---|
//! | `object.rs` | `object.h`: the transforms of an object, the ray into an instance |
//! | `triangle/` | `triangle.h`, `triangle_intersect.h`: the corners, the normals, a ray against one |
//! | `shader_data.rs` | `shader_data.h`: the shading point of a hit |
mod object;
mod shader_data;
pub mod triangle;
pub use object::{ObjectTransform, bvh_clamp_direction, bvh_instance_pop, bvh_instance_push, bvh_inverse_direction, object_fetch_transform, object_inverse_dir_transform, object_normal_transform, object_position_transform};
pub use shader_data::{shader_setup_from_background, shader_setup_from_ray};
pub use triangle::{triangle_intersect, triangle_normal, triangle_smooth_normal, triangle_vertices};
