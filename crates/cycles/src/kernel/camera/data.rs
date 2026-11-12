// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h` (`CameraType`, line 539;
//! `KernelCamera`, lines 1162-1250): the camera as a kernel reads it. The
//! scene's `Camera` writes it into the constant data; this is the members a
//! ray needs, taken out of those words once.

use crate::kernel::{KernelData, constant};
use crate::util::{ProjectionTransform, Transform};

/// `CameraType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraType {
    Perspective,
    Orthographic,
    Panorama,
    Custom,
}

impl CameraType {
    /// The type `kernel_data.cam.type` names, by Cycles' numbers.
    pub fn of(number: i32) -> Option<CameraType> {
        let name = [("CAMERA_PERSPECTIVE", Self::Perspective), ("CAMERA_ORTHOGRAPHIC", Self::Orthographic), ("CAMERA_PANORAMA", Self::Panorama), ("CAMERA_CUSTOM", Self::Custom)];
        name.into_iter().find(|(name, _)| constant(name) == Some(i64::from(number))).map(|(_, kind)| kind)
    }
}

/// `KernelCamera`, the members the ported kernels read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KernelCamera {
    /// `type`.
    pub kind: Option<CameraType>,
    pub rastertocamera: ProjectionTransform,
    pub cameratoworld: Transform,
    /// The raster to the camera at the shutter's open and close, for a
    /// field of view that changes.
    pub perspective_pre: ProjectionTransform,
    pub perspective_post: ProjectionTransform,
    pub have_perspective_motion: bool,
    /// `num_motion_steps`: how many of `camera_motion` there are.
    pub motion_step_count: usize,
    pub aperturesize: f32,
    pub focaldistance: f32,
    pub blades: f32,
    pub bladesrotation: f32,
    pub inv_aperture_ratio: f32,
    pub interocular_offset: f32,
    pub nearclip: f32,
    pub cliplength: f32,
    /// One pixel across and one down, in the world.
    pub dx: [f32; 3],
    pub dy: [f32; 3],
    pub differential_scale: f32,
    /// -1 without motion blur.
    pub shuttertime: f32,
    pub shutter_table_offset: usize,
    pub rolling_shutter_type: i32,
    pub rolling_shutter_duration: f32,
    /// The image in pixels.
    pub width: f32,
    pub height: f32,
}

impl KernelCamera {
    /// `kernel_data.cam`.
    pub fn of(data: &KernelData) -> KernelCamera {
        let float = |name: &str| data.float("cam", name);
        let (dx, dy) = (data.float4("cam", "dx"), data.float4("cam", "dy"));
        KernelCamera {
            kind: CameraType::of(data.int("cam", "type")),
            rastertocamera: data.projection("cam", "rastertocamera"),
            cameratoworld: data.transform("cam", "cameratoworld"),
            perspective_pre: data.projection("cam", "perspective_pre"),
            perspective_post: data.projection("cam", "perspective_post"),
            have_perspective_motion: data.int("cam", "have_perspective_motion") != 0,
            motion_step_count: data.int("cam", "num_motion_steps").max(0) as usize,
            aperturesize: float("aperturesize"),
            focaldistance: float("focaldistance"),
            blades: float("blades"),
            bladesrotation: float("bladesrotation"),
            inv_aperture_ratio: float("inv_aperture_ratio"),
            interocular_offset: float("interocular_offset"),
            nearclip: float("nearclip"),
            cliplength: float("cliplength"),
            dx: [dx[0], dx[1], dx[2]],
            dy: [dy[0], dy[1], dy[2]],
            differential_scale: float("differential_scale"),
            shuttertime: float("shuttertime"),
            shutter_table_offset: data.int("cam", "shutter_table_offset").max(0) as usize,
            rolling_shutter_type: data.int("cam", "rolling_shutter_type"),
            rolling_shutter_duration: float("rolling_shutter_duration"),
            width: float("width"),
            height: float("height"),
        }
    }
}

#[cfg(test)]
mod test;
