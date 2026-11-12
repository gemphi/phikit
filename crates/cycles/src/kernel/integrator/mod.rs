// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/integrator/`: the kernels a path
//! moves between, each doing one thing to it and naming the next.
//!
//! | File | Ported from |
//! |---|---|
//! | `state.rs`, `layout.rs` | `state_template.h`, `shadow_state_template.h`, `state.h` |
//! | `path_state.rs` | `path_state.h`, and the control flow of `state_flow.h` |
//! | `init_from_camera.rs` | `init_from_camera.h` |
//! | `intersect_closest.rs` | `intersect_closest.h` |
//! | `shade_background.rs` | `shade_background.h` |
//! | `shade_surface.rs` | `shade_surface.h`, `surface_shader.h` |
//! | `megakernel.rs` | `megakernel.h` |
mod init_from_camera;
mod intersect_closest;
mod layout;
mod megakernel;
mod path_state;
mod shade_background;
mod shade_surface;
mod state;
pub use init_from_camera::integrator_init_from_camera;
pub use intersect_closest::integrator_intersect_closest;
pub use layout::{StateLayout, StateMember};
pub use megakernel::integrator_megakernel;
pub use path_state::{RngState, integrator_path_init, integrator_path_next, integrator_path_terminate, path_state_continuation_probability, path_state_init, path_state_init_integrator, path_state_next, path_state_ray_visibility, path_state_rng_1d, path_state_rng_2d, path_state_rng_3d, path_state_rng_load};
pub use shade_background::{integrator_eval_background_shader, integrator_shade_background, is_light_shader_visible_to_path};
pub use shade_surface::integrator_shade_surface;
pub use state::{IntegratorState, IsectState, PathState, RayState, ShadowPathState};
