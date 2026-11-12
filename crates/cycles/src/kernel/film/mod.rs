// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/film/`: what a path writes to the
//! render buffer. The combined pass only (an opaque film or a film with a
//! transparent background); light passes, data passes, AOVs, the shadow
//! catcher and adaptive sampling are not ported.
//!
//! | File | Ported from |
//! |---|---|
//! | `write.rs` | `write.h`: the pixel of a path, adding to a pass |
//! | `light_passes.rs` | `light_passes.h`: the combined pass, background, emission, clamping |
mod light_passes;
mod write;
pub use light_passes::{film_clamp_light, film_write_background, film_write_combined_transparent_pass, film_write_sample, film_write_surface_emission};
pub use write::{film_pass_pixel_render_buffer, film_write_pass_float4, film_write_pass_spectrum};
