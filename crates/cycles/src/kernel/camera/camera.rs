// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/camera/camera.h`:
//! `camera_sample_perspective` (line 46), `camera_sample_orthographic`
//! (155) and `camera_sample` (the pixel filter, line 470).
use crate::kernel::globals::KernelGlobals;
use crate::kernel::sample::sample_uniform_disk;
use crate::kernel::types::Ray;
use crate::util::vector::{add, normalize, scale, sub};

/// `camera_sample_aperture` for a circular aperture, stretched by the
/// anamorphic ratio.
fn camera_sample_aperture(kg: &KernelGlobals, rand: [f32; 2]) -> [f32; 2] {
    let bokeh = sample_uniform_disk(rand);
    [bokeh[0] * kg.cam.inv_aperture_ratio, bokeh[1]]
}

/// `camera_sample_perspective`: a ray from the origin through the raster
/// position, bent about the plane of focus when there is a lens.
pub fn camera_sample_perspective(kg: &KernelGlobals, raster: [f32; 2], rand_lens: [f32; 2], ray: &mut Ray) -> [f32; 3] {
    let cam = &kg.cam;
    let pcamera = cam.rastertocamera.point([raster[0], raster[1], 0.0]);
    let mut p = [0.0; 3];
    let mut d = pcamera;
    // Depth of field.
    if cam.aperturesize > 0.0 {
        let lens_uv = camera_sample_aperture(kg, rand_lens).map(|part| part * cam.aperturesize);
        let ft = cam.focaldistance / d[2];
        let pfocus = scale(d, ft);
        p = [lens_uv[0], lens_uv[1], 0.0];
        d = normalize(sub(pfocus, p));
    }
    // Camera to world.
    let tfm = &cam.cameratoworld;
    p = tfm.point(p);
    d = normalize(tfm.direction(d));
    ray.p = p;
    ray.d = d;
    // Ray differentials, as one number each (`differential_make_compact`
    // of the two directions' lengths).
    let dcenter = tfm.direction(pcamera);
    let dcenter_normalized = normalize(dcenter);
    let dx = sub(normalize(add(dcenter, cam.dx)), dcenter_normalized);
    let dy = sub(normalize(add(dcenter, cam.dy)), dcenter_normalized);
    ray.dp = 0.0;
    ray.dd = 0.5 * (len(dx) + len(dy)) * cam.differential_scale;
    // Clipping.
    let z_inv = 1.0 / normalize(pcamera)[2];
    let nearclip = cam.nearclip * z_inv;
    ray.p = add(ray.p, scale(ray.d, nearclip));
    ray.tmin = 0.0;
    ray.tmax = cam.cliplength * z_inv;
    [1.0; 3]
}

fn len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// `camera_sample_orthographic`: parallel rays, from the raster position.
pub fn camera_sample_orthographic(kg: &KernelGlobals, raster: [f32; 2], rand_lens: [f32; 2], ray: &mut Ray) -> [f32; 3] {
    let cam = &kg.cam;
    let pcamera = cam.rastertocamera.point([raster[0], raster[1], 0.0]);
    let mut d = [0.0, 0.0, 1.0];
    let p;
    if cam.aperturesize > 0.0 {
        let lens_uv = camera_sample_aperture(kg, rand_lens).map(|part| part * cam.aperturesize);
        let pfocus = scale(d, cam.focaldistance);
        let lens_uvw = [lens_uv[0], lens_uv[1], 0.0];
        d = normalize(sub(pfocus, lens_uvw));
        // Where the ray is when it reaches the near clip plane.
        p = add(add(pcamera, lens_uvw), scale(d, cam.nearclip / d[2]));
    } else {
        p = add(pcamera, [0.0, 0.0, cam.nearclip]);
    }
    let tfm = &cam.cameratoworld;
    ray.p = tfm.point(p);
    ray.d = normalize(tfm.direction(d));
    ray.dp = 0.5 * (len(cam.dx) + len(cam.dy)) * cam.differential_scale;
    ray.dd = 0.0;
    ray.tmin = 0.0;
    ray.tmax = cam.cliplength;
    [1.0; 3]
}

/// `camera_sample`: the ray through pixel `(x, y)`, moved inside it by the
/// pixel filter's table (`filter_uv` in 0..1) or by the pixel jitter. The
/// throughput the ray starts with (the camera's sensitivity) is returned.
pub fn camera_sample(kg: &KernelGlobals, x: i32, y: i32, filter_uv: [f32; 2], time: f32, lens_uv: [f32; 2], ray: &mut Ray) -> [f32; 3] {
    let mut raster = [x as f32, y as f32];
    if kg.integrator.pixel_jitter[0] == f32::MAX {
        let size = crate::kernel::constant("FILTER_TABLE_SIZE").unwrap_or(1024) as i32;
        raster[0] += kg.lookup_table_read(filter_uv[0], kg.tables_filter_offset, size);
        raster[1] += kg.lookup_table_read(filter_uv[1], kg.tables_filter_offset, size);
    } else {
        raster = [raster[0] - kg.integrator.pixel_jitter[0], raster[1] - kg.integrator.pixel_jitter[1]];
    }
    // No motion blur: the middle of the shutter.
    ray.time = 0.5;
    let _ = time;
    match kg.cam.kind == KernelGlobals::camera_kind("CAMERA_ORTHOGRAPHIC") {
        true => camera_sample_orthographic(kg, raster, lens_uv, ray),
        false => camera_sample_perspective(kg, raster, lens_uv, ray),
    }
}
