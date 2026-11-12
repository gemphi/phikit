// SPDX-FileCopyrightText: 2009-2010 Sony Pictures Imageworks Inc., et al. All Rights Reserved.
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/closure/`: the closures a shader
//! allocates and what a path does with them.
//!
//! | File | Ported from |
//! |---|---|
//! | `alloc.rs` | `alloc.h`: `closure_alloc`, `bsdf_alloc` |
//! | `diffuse.rs` | `bsdf_diffuse.h`: the diffuse BSDF |
//! | `emissive.rs` | `emissive.h`: the emission and background closures |
//! | `bsdf.rs` | `bsdf.h`: `bsdf_sample`, for the closures ported |
mod alloc;
mod bsdf;
mod diffuse;
mod emissive;
pub use alloc::{bsdf_alloc, closure_alloc, closure_sample_weight};
pub use bsdf::{BsdfSample, bsdf_sample, closure_is_bsdf};
pub use diffuse::{bsdf_diffuse_eval, bsdf_diffuse_sample, bsdf_diffuse_setup};
pub use emissive::{background_setup, emission_setup, emissive_pdf, emissive_simple_eval};
