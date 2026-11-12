// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/camera/camera.h`
//! (`camera_sample_perspective`, lines 43-173), without the spherical
//! stereo branch (lines 121-162), which comes with the panorama.

use super::{camera_sample_aperture, camera_to_world};
use crate::kernel::util::{Differential3, differential_make_compact, differential_zero_compact};
use crate::kernel::{KernelGlobals, Ray};
use crate::util::vector::{add, interp, normalize, scale, sub};

/// `camera_sample_perspective`: the ray of a perspective camera through
/// `raster_xy`, a place in the image in pixels. `ray.time` is read; the
/// rest of the ray is written. The answer is what the camera lets through.
pub fn camera_sample_perspective(kg: &KernelGlobals, raster_xy: [f32; 2], rand_lens: [f32; 2], ray: &mut Ray) -> [f32; 3] {
    let cam = &kg.cam;
    let raster = [raster_xy[0], raster_xy[1], 0.0];
    let mut pcamera = cam.rastertocamera.point(raster);
    // A field of view that changes over the shutter: the projected point
    // is mixed, which is simple and looks right, not the field of view.
    if cam.have_perspective_motion {
        pcamera = match ray.time < 0.5 {
            true => interp(cam.perspective_pre.point(raster), pcamera, ray.time * 2.0),
            false => interp(pcamera, cam.perspective_post.point(raster), (ray.time - 0.5) * 2.0),
        };
    }

    // Depth of field: start on the lens, aim at where the pixel is sharp.
    let (mut p, mut d) = ([0.0; 3], pcamera);
    if cam.aperturesize > 0.0 {
        let lens = scale(camera_sample_aperture(cam, rand_lens), cam.aperturesize);
        let focus = scale(d, cam.focaldistance / d[2]);
        p = [lens[0], lens[1], 0.0];
        d = normalize(sub(focus, p));
    }

    let cameratoworld = camera_to_world(kg, ray.time);
    ray.p = cameratoworld.point(p);
    ray.d = normalize(cameratoworld.direction(d));

    // How far the rays of the pixels beside this one are, by the middle of
    // the lens.
    let center = cameratoworld.direction(pcamera);
    let center_normalized = normalize(center);
    let change = Differential3 { dx: sub(normalize(add(center, cam.dx)), center_normalized), dy: sub(normalize(add(center, cam.dy)), center_normalized) };
    ray.dp = differential_zero_compact();
    ray.dd = differential_make_compact(&change) * cam.differential_scale;

    // Clipping: the planes are flat, so a ray off the axis goes further.
    let z_inverse = 1.0 / normalize(pcamera)[2];
    let nearclip = cam.nearclip * z_inverse;
    ray.p = add(ray.p, scale(ray.d, nearclip));
    ray.dp += nearclip * ray.dd;
    ray.tmin = 0.0;
    ray.tmax = cam.cliplength * z_inverse;
    [1.0; 3]
}
