// SPDX-License-Identifier: Apache-2.0
//! Random numbers made into what a kernel samples:
//! `intern/cycles/kernel/sample`. So far the two a lens needs.

mod mapping;

pub use mapping::{regular_polygon_sample, sample_uniform_disk};
