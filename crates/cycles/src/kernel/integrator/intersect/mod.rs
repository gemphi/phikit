// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! A path's ray traced: `kernel/integrator/intersect_*.h`. `closest.rs` is
//! `intersect_closest.h`; the shadow, subsurface, volume stack and
//! dedicated light walks come with what needs them.
mod closest;
pub use closest::integrator_intersect_closest;
