// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/camera/camera.h`
//! (`camera_sample_orthographic`, lines 176-237).

use super::{camera_sample_aperture, camera_to_world};
use crate::kernel::util::{Differential3, differential_make_compact, differential_zero_compact};
use crate::kernel::{KernelGlobals, Ray};
use crate::util::vector::{add, normalize, scale, sub};

/// `camera_sample_orthographic`: the ray of an orthographic camera through
/// `raster_xy`. Every ray goes the same way; they start side by side.
pub fn camera_sample_orthographic(kg: &KernelGlobals, raster_xy: [f32; 2], rand_lens: [f32; 2], ray: &mut Ray) -> [f32; 3] {
    let cam = &kg.cam;
    let pcamera = cam.rastertocamera.point([raster_xy[0], raster_xy[1], 0.0]);
    let forward = [0.0, 0.0, 1.0];
    let (p, d) = match cam.aperturesize > 0.0 {
        true => {
            let lens = scale(camera_sample_aperture(cam, rand_lens), cam.aperturesize);
            let lens = [lens[0], lens[1], 0.0];
            let d = normalize(sub(scale(forward, cam.focaldistance), lens));
            // Where the ray is when it crosses the near plane, so that
            // clipping does not move what is sharp.
            (add(add(pcamera, lens), scale(d, cam.nearclip / d[2])), d)
        }
        false => (add(pcamera, [0.0, 0.0, cam.nearclip]), forward),
    };

    let cameratoworld = camera_to_world(kg, ray.time);
    ray.p = cameratoworld.point(p);
    ray.d = normalize(cameratoworld.direction(d));
    ray.dp = differential_make_compact(&Differential3 { dx: cam.dx, dy: cam.dy }) * cam.differential_scale;
    ray.dd = differential_zero_compact();
    ray.tmin = 0.0;
    ray.tmax = cam.cliplength;
    [1.0; 3]
}
