// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/camera/camera.h`: the ray a camera
//! sends through a pixel. Here are `camera_sample_aperture` (lines 22-41),
//! `camera_sample` (lines 449-527), `camera_position` and
//! `camera_distance` (lines 531-547); the perspective and the orthographic
//! camera have a file each.
//!
//! Not ported: the panorama camera and spherical stereo
//! (`kernel/camera/projection.h`), and the custom camera, which is an OSL
//! shader. `camera_sample` answers black for them, as Cycles does for a ray
//! that could not be made.

mod data;
mod orthographic;
mod perspective;

pub use data::{CameraType, KernelCamera};
pub use orthographic::camera_sample_orthographic;
pub use perspective::camera_sample_perspective;

use super::sample::{regular_polygon_sample, sample_uniform_disk};
use super::util::lookup_table_read;
use super::{KernelGlobals, Ray};
use crate::util::vector::{dot, len, sub};
use crate::util::{Transform, motion_array_interpolate};

/// `camera_sample_aperture`: a point of the lens, a disk or a polygon of
/// blades, squeezed across for an anamorphic lens.
pub fn camera_sample_aperture(cam: &KernelCamera, rand: [f32; 2]) -> [f32; 2] {
    let bokeh = match cam.blades == 0.0 {
        true => sample_uniform_disk(rand),
        false => regular_polygon_sample(cam.blades, cam.bladesrotation, rand),
    };
    [bokeh[0] * cam.inv_aperture_ratio, bokeh[1]]
}

/// The camera in the world at `time`: where it is, or for a camera that
/// moves, where it is at that moment of the shutter.
pub fn camera_to_world(kg: &KernelGlobals, time: f32) -> Transform {
    let step = kg.cam.motion_step_count.min(kg.scene.camera_motion.len());
    match step {
        0 => kg.cam.cameratoworld,
        _ => motion_array_interpolate(&kg.scene.camera_motion[..step], time),
    }
}

/// The moment of the shutter a ray is sent at, from the random number
/// `time`: the shutter's curve, and for a rolling shutter the row.
fn shutter_time(kg: &KernelGlobals, y: i32, time: f32) -> f32 {
    let cam = &kg.cam;
    if cam.shuttertime == -1.0 {
        return 0.5;
    }
    let open = lookup_table_read(&kg.scene.lookup_table, time, cam.shutter_table_offset, kg.shutter_table_size);
    if cam.rolling_shutter_type == 0 {
        return open;
    }
    // A shutter that only rolls: the top row at 0, the bottom row at 1.
    let row = 1.0 - y as f32 / cam.height;
    let duration = cam.rolling_shutter_duration;
    // Not physically right, but simple to control: between a frame that
    // only rolls and one that only blurs.
    match duration != 0.0 {
        true => (open - 0.5) * duration + (row - 0.5) * (1.0 - duration) + 0.5,
        false => row,
    }
}

/// `camera_sample`: the ray of pixel `x`, `y` for the random numbers of
/// one sample, written into `ray`. The answer is what the camera lets
/// through, which a path starts its throughput with: black when no ray
/// could be made.
pub fn camera_sample(kg: &KernelGlobals, x: i32, y: i32, filter_uv: [f32; 2], time: f32, lens_uv: [f32; 2], ray: &mut Ray) -> [f32; 3] {
    // The pixel filter: a place within the pixel's reach, or a fixed one.
    let filter = |u: f32| lookup_table_read(&kg.scene.lookup_table, u, kg.filter_table_offset, kg.filter_table_size);
    let offset = match kg.integrator.pixel_jitter[0] == f32::MAX {
        true => filter_uv.map(filter),
        false => kg.integrator.pixel_jitter.map(|jitter| -jitter),
    };
    let raster = [x as f32 + offset[0], y as f32 + offset[1]];
    ray.time = shutter_time(kg, y, time);
    let stereo = kg.cam.interocular_offset != 0.0;
    match (kg.cam.kind, stereo) {
        (Some(CameraType::Perspective), false) => camera_sample_perspective(kg, raster, lens_uv, ray),
        (Some(CameraType::Orthographic), _) => camera_sample_orthographic(kg, raster, lens_uv, ray),
        _ => [0.0; 3],
    }
}

/// `camera_position`: where the camera is in the world.
pub fn camera_position(kg: &KernelGlobals) -> [f32; 3] {
    kg.cam.cameratoworld.column(3)
}

/// `camera_distance`: how far `p` is from the camera; from its plane for
/// an orthographic camera.
pub fn camera_distance(kg: &KernelGlobals, p: [f32; 3]) -> f32 {
    let from = sub(p, camera_position(kg));
    match kg.cam.kind {
        Some(CameraType::Orthographic) => dot(from, kg.cam.cameratoworld.column(2)).abs(),
        _ => len(from),
    }
}
