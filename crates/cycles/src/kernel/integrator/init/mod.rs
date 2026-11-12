// SPDX-License-Identifier: Apache-2.0
//! Where a path starts: `kernel/integrator/init_from_*.h`. `camera.rs` is
//! `init_from_camera.h`; `init_from_bake.h` comes with baking.
mod camera;
pub use camera::integrator_init_from_camera;
