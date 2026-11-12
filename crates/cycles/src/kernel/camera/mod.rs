// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/camera/`: the ray of a pixel.
//! `camera.rs` is `camera.h`; the perspective and orthographic cameras
//! only. Panorama and custom cameras, a polygonal aperture, spherical
//! stereo, motion blur and rolling shutter are not ported.
mod camera;
pub use camera::{camera_sample, camera_sample_orthographic, camera_sample_perspective};
