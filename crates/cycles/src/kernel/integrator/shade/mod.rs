// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! What a path's ray met, shaded: `kernel/integrator/shade_*.h`.
//! `background.rs` is `shade_background.h`, `surface.rs` is
//! `shade_surface.h` with `surface_shader.h`; lights, shadows and volumes
//! come with gate 3.
mod background;
mod surface;
pub use background::{integrator_eval_background_shader, integrator_shade_background, is_light_shader_visible_to_path};
pub use surface::integrator_shade_surface;
