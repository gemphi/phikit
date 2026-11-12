// SPDX-License-Identifier: Apache-2.0
//! Triangles: `intern/cycles/kernel/geom/triangle*.h`. So far a ray
//! against one.

mod intersect;

pub use intersect::triangle_intersect;
