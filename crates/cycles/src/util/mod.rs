// SPDX-License-Identifier: Apache-2.0
//! What every layer leans on: `intern/cycles/util`. So far the clock the
//! scheduler reads, integer helpers, hashes, vectors, transforms, boxes and
//! the reader of the data files.

mod bound;
mod cdf;
mod hash;
mod math;
mod md5;
mod motion;
mod murmur;
mod normal;
mod param;
mod projection;
mod table;
mod time;
mod transform;
pub mod vector;

pub use bound::BoundBox;
pub use cdf::{cdf_evaluate, cdf_inverted, cdf_invert};
pub use hash::{hash_uint, hash_uint2, hash_uint3};
pub use md5::Md5Hash;
pub use math::{align_up, divide_up, is_power_of_two, next_power_of_two, reverse_integer_bits};
pub use motion::{DecomposedTransform, compose, motion_array_interpolate, motion_decompose};
pub use murmur::{hash_to_float, murmur_hash3};
pub use normal::PackedNormal;
pub use param::param_default;
pub use projection::ProjectionTransform;
pub use table::row_list;
pub use transform::{BoundBox2D, Transform};
#[cfg(not(target_family = "wasm"))]
pub use time::SystemClock;
pub use time::{Clock, ManualClock};
