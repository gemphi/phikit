// SPDX-License-Identifier: Apache-2.0
//! The shapes a ray meets: `intern/cycles/kernel/geom`. So far an object's
//! place, for a ray that enters an instance, and a triangle hit.

mod object;
pub mod triangle;

pub use object::{ObjectField, ObjectTransform, bvh_clamp_direction, bvh_instance_pop, bvh_instance_push, bvh_inverse_direction, object_fetch_transform};
