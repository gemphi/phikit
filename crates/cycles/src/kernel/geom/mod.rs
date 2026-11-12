// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/geom/`: reading the geometry of a
//! hit out of the device scene.
//!
//! | File | Ported from |
//! |---|---|
//! | `object.rs` | `object.h`: the transforms of an object, the ray into an instance |
//! | `triangle.rs` | `triangle.h`, `triangle_intersect.h`: the points, the normals |
//! | `shader_data.rs` | `shader_data.h`: the shading point of a hit |
mod object;
mod shader_data;
mod triangle;
pub use object::{bvh_clamp_direction, bvh_inverse_direction, object_inverse_dir_transform, object_normal_transform, object_position_transform};
pub use shader_data::{shader_setup_from_background, shader_setup_from_ray};
pub use triangle::{triangle_intersect, triangle_normal, triangle_smooth_normal, triangle_vertices};
