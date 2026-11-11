// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/blender/camera.cpp` (`blender_camera_viewplane`
//! lines 399-446, `blender_camera_matrix` 368-396, `blender_camera_sync`
//! 550-680): a Blender camera into Cycles' `Camera`. The perspective and
//! orthographic cameras only; panoramas, depth of field, motion and the
//! camera border come with the scenes that use them.
use super::source::{CameraSource, SensorFit};
use crate::graph::Value;
use crate::scene::Camera;
use crate::util::{BoundBox2D, Transform};

/// Blender's `float[4][4]` (`matrix[column][row]`) as the rows of a
/// Cycles `Transform` (`get_transform` of `blender/util.h`).
pub fn transform_of(matrix: &[[f32; 4]; 4]) -> Transform {
    let row = |r: usize| [matrix[0][r], matrix[1][r], matrix[2][r], matrix[3][r]];
    Transform { x: row(0), y: row(1), z: row(2) }
}

/// `blender_camera_matrix` for a perspective or orthographic camera: the
/// Blender camera looks down -Z, Cycles' down +Z, and the scale goes.
pub fn camera_matrix(matrix: &[[f32; 4]; 4]) -> Transform {
    (transform_of(matrix) * Transform::scale(1.0, 1.0, -1.0)).clear_scale()
}

/// `blender_camera_viewplane`: the viewplane, the aspect ratio of the
/// fitted side, and the sensor size on that side.
pub fn viewplane(source: &CameraSource, width: i32, height: i32) -> (BoundBox2D, f32, f32) {
    let (xratio, yratio) = (width as f32, height as f32);
    let (horizontal, sensor_size) = match source.sensor_fit {
        SensorFit::Auto => (xratio > yratio, source.sensor_width),
        SensorFit::Horizontal => (true, source.sensor_width),
        SensorFit::Vertical => (false, source.sensor_height),
    };
    let mut aspectratio = if horizontal { xratio / yratio } else { yratio / xratio };
    let mut aspect = if horizontal { [aspectratio, 1.0] } else { [1.0, aspectratio] };
    if source.orthographic {
        let scale = source.ortho_scale / (aspectratio * 2.0);
        aspect = aspect.map(|part| part * scale);
        aspectratio = source.ortho_scale / 2.0;
    }
    // Camera shift, with no 3D view offset and a zoom of 1.
    let dv = [2.0 * aspectratio * source.shift[0], 2.0 * aspectratio * source.shift[1]];
    let plane = BoundBox2D { left: -aspect[0] + dv[0], right: aspect[0] + dv[0], bottom: -aspect[1] + dv[1], top: aspect[1] + dv[1] };
    (plane, aspectratio, sensor_size)
}

/// `blender_camera_sync`: a `Camera` for `source` at `width` by `height`.
pub fn sync_camera(source: &CameraSource, width: i32, height: i32) -> Camera {
    let mut camera = Camera::new();
    let (plane, aspectratio, sensor_size) = viewplane(source, width, height);
    camera.set_bound("viewplane", &plane);
    let node = &mut camera.node;
    node.set("full_width", Value::Int(width));
    node.set("full_height", Value::Int(height));
    node.set("nearclip", Value::Float(source.clip_start));
    node.set("farclip", Value::Float(source.clip_end));
    node.set_enum("camera_type", if source.orthographic { "orthograph" } else { "perspective" });
    node.set("fov", Value::Float(2.0 * ((0.5 * sensor_size) / source.lens / aspectratio).atan()));
    node.set("matrix", Value::Transform(camera_matrix(&source.matrix)));
    camera.set_screen_size(width, height);
    camera
}
