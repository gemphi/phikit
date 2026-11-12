// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/integrator/`: the kernels a path
//! moves between, each doing one thing to it and naming the next. Cycles
//! names a kernel's file by what it does and to what (`init_from_camera.h`,
//! `shade_surface.h`); here the doing is the folder and the what the file.
//!
//! | File | Ported from |
//! |---|---|
//! | `state.rs`, `layout.rs` | `state_template.h`, `shadow_state_template.h`, `state.h` |
//! | `path.rs` | `path_state.h`, and the control flow of `state_flow.h` |
//! | `init/camera.rs` | `init_from_camera.h` |
//! | `intersect/closest.rs` | `intersect_closest.h` |
//! | `shade/background.rs` | `shade_background.h` |
//! | `shade/surface.rs` | `shade_surface.h`, `surface_shader.h` |
//! | `megakernel.rs` | `megakernel.h` |
mod init;
mod intersect;
mod layout;
mod megakernel;
mod path;
mod shade;
mod state;
pub use init::integrator_init_from_camera;
pub use intersect::integrator_intersect_closest;
pub use layout::{StateLayout, StateMember};
pub use megakernel::integrator_megakernel;
pub use path::{RngState, integrator_path_init, integrator_path_next, integrator_path_terminate, path_state_continuation_probability, path_state_init, path_state_init_integrator, path_state_next, path_state_ray_visibility, path_state_rng_1d, path_state_rng_2d, path_state_rng_3d, path_state_rng_load};
pub use shade::{integrator_eval_background_shader, integrator_shade_background, integrator_shade_surface, is_light_shader_visible_to_path};
pub use state::{IntegratorState, IsectState, PathState, RayState, ShadowPathState};

#[cfg(test)]
mod test;
